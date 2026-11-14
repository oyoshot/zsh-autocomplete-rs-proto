use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use super::{Result, interrupted};

pub struct Shell {
    child: Child,
    master: File,
    output: Vec<u8>,
    pub started: Instant,
}

impl Shell {
    pub fn spawn(command: &mut Command) -> Result<Self> {
        let started = Instant::now();
        let (mut master, mut slave) = (-1, -1);
        let mut size = libc::winsize {
            ws_row: 24,
            ws_col: 80,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        // SAFETY: all non-null pointers refer to live writable values. Apple
        // declares the size pointer mutable; Linux accepts the same raw pointer.
        if unsafe {
            libc::openpty(
                &mut master,
                &mut slave,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &raw mut size,
            )
        } != 0
        {
            return Err(std::io::Error::last_os_error().into());
        }
        // SAFETY: successful openpty transfers ownership of these distinct fds.
        let (master, slave) = unsafe { (File::from_raw_fd(master), File::from_raw_fd(slave)) };
        for file in [&master, &slave] {
            // Do not leak extra terminal descriptors into the shell or daemon.
            if unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETFD, libc::FD_CLOEXEC) } < 0 {
                return Err(std::io::Error::last_os_error().into());
            }
        }
        command
            .stdin(Stdio::from(slave.try_clone()?))
            .stdout(Stdio::from(slave.try_clone()?))
            .stderr(Stdio::from(slave));
        // SAFETY: only async-signal-safe syscalls run between fork and exec.
        unsafe {
            command.pre_exec(|| {
                if libc::setsid() < 0 || libc::ioctl(0, libc::TIOCSCTTY as _, 0) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let child = command.spawn()?;
        Ok(Self {
            child,
            master,
            output: Vec::new(),
            started,
        })
    }

    fn drain(&mut self, timeout_ms: i32) -> Result<()> {
        let mut fd = libc::pollfd {
            fd: self.master.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: fd points to one live pollfd for the duration of the call.
        let result = unsafe { libc::poll(&mut fd, 1, timeout_ms) };
        if result < 0 {
            let error = std::io::Error::last_os_error();
            if error.kind() == std::io::ErrorKind::Interrupted {
                return Ok(());
            }
            return Err(error.into());
        }
        if result > 0 {
            let mut buffer = [0; 8192];
            match self.master.read(&mut buffer) {
                Ok(0) | Err(_) => {
                    return Err(format!(
                        "Zsh closed before probe: {}",
                        String::from_utf8_lossy(&self.output)
                    )
                    .into());
                }
                Ok(n) => self.output.extend_from_slice(&buffer[..n]),
            }
            if self.output.len() > 65536 {
                self.output.drain(..self.output.len() - 65536);
            }
        }
        Ok(())
    }

    pub fn ready(&mut self) -> Result<f64> {
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            interrupted()?;
            self.drain(100)?;
            // Terminal escapes can precede the marker, and reads can split it.
            let text = String::from_utf8_lossy(&self.output);
            if let Some((_, rest)) = text.split_once("__ZACRS_READY__:")
                && let Some((value, _)) = rest.split_once('\n')
            {
                let value: f64 = value.trim().parse()?;
                if value.is_finite() && value >= 0.0 {
                    return Ok(value);
                }
                return Err("invalid source timing".into());
            }
        }
        Err(format!(
            "readiness timeout: {}",
            String::from_utf8_lossy(&self.output)
        )
        .into())
    }

    pub fn probe(&mut self, path: &std::path::Path, expected: &str) -> Result<()> {
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            interrupted()?;
            self.drain(0)?;
            if fs::read_to_string(path).is_ok_and(|text| text == expected) {
                return Ok(());
            }
            std::thread::sleep(Duration::from_micros(200));
        }
        Err(format!("probe timeout: {}", path.display()).into())
    }

    pub fn send(&mut self, bytes: &[u8]) -> Result<()> {
        self.master.write_all(bytes)?;
        Ok(())
    }
}

impl Drop for Shell {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
