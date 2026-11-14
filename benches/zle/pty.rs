use portable_pty::{Child, CommandBuilder, MasterPty};
use std::fs;
use std::io::{Read, Write};
use std::process::Command;
use std::time::{Duration, Instant};

use super::{Result, interrupted};

pub struct Shell {
    child: Box<dyn Child + Send + Sync>,
    master: Box<dyn MasterPty + Send>,
    reader: Box<dyn Read + Send>,
    writer: Box<dyn Write + Send>,
    output: Vec<u8>,
    pub started: Instant,
}

impl Shell {
    pub fn spawn(command: &mut Command) -> Result<Self> {
        let started = Instant::now();
        let pair = portable_pty::native_pty_system().openpty(portable_pty::PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;
        let mut builder = CommandBuilder::new(command.get_program());
        builder.args(command.get_args());
        builder.env_clear();
        for (name, value) in command.get_envs() {
            if let Some(value) = value {
                builder.env(name, value);
            }
        }
        builder.cwd(std::env::current_dir()?);
        let child = pair.slave.spawn_command(builder)?;
        drop(pair.slave);
        Ok(Self {
            child,
            master: pair.master,
            reader,
            writer,
            output: Vec::new(),
            started,
        })
    }

    fn drain(&mut self, timeout_ms: i32) -> Result<()> {
        let mut fd = filedescriptor::pollfd {
            fd: self
                .master
                .as_raw_fd()
                .ok_or("PTY has no Unix descriptor")?,
            events: filedescriptor::POLLIN,
            revents: 0,
        };
        let result = filedescriptor::poll(
            std::slice::from_mut(&mut fd),
            Some(Duration::from_millis(timeout_ms as u64)),
        );
        interrupted()?;
        if result? > 0 {
            let mut buffer = [0; 8192];
            match self.reader.read(&mut buffer) {
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
        self.writer.write_all(bytes)?;
        Ok(())
    }
}

impl Drop for Shell {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
