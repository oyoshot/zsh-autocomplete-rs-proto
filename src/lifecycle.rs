//! Explicit daemon startup and shutdown. Shell startup only checks capabilities.
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::protocol;

const TIMEOUT: Duration = Duration::from_secs(1);
const WAIT_LIMIT: Duration = Duration::from_secs(3);

fn timed_out() -> io::Error {
    io::Error::new(io::ErrorKind::TimedOut, "timed out waiting for daemon")
}

fn lock(path: &Path) -> io::Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)?;
    let deadline = Instant::now() + WAIT_LIMIT;
    loop {
        // The descriptor remains open for the entire replacement transaction.
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0 {
            return Ok(file);
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::WouldBlock {
            return Err(error);
        }
        if Instant::now() >= deadline {
            return Err(timed_out());
        }
        thread::sleep(Duration::from_millis(5));
    }
}

fn request(path: &Path, message: &[u8]) -> io::Result<String> {
    let mut stream = UnixStream::connect(path)?;
    stream.set_read_timeout(Some(TIMEOUT))?;
    stream.set_write_timeout(Some(TIMEOUT))?;
    stream.write_all(message)?;
    let mut line = String::new();
    BufReader::new(stream).take(4096).read_line(&mut line)?;
    Ok(line.trim_end().to_owned())
}

fn probe(path: &Path) -> io::Result<Option<String>> {
    let reply = match request(path, b"ping shell\n") {
        Ok(reply) => reply,
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused
            ) =>
        {
            return Ok(None);
        }
        Err(error) => return Err(error),
    };
    if reply.starts_with("OK") {
        return Ok(Some(reply));
    }
    // Before shell capabilities existed, unknown requests closed the socket.
    let reply = request(path, b"ping\n")?;
    if reply == "OK" {
        Ok(Some(reply))
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid daemon handshake",
        ))
    }
}

fn shutdown(path: &Path) -> io::Result<()> {
    if request(path, b"shutdown\n")? != "OK" {
        return Err(io::Error::other("daemon refused shutdown"));
    }
    let deadline = Instant::now() + WAIT_LIMIT;
    while path.exists() {
        if Instant::now() >= deadline {
            return Err(timed_out());
        }
        thread::sleep(Duration::from_millis(2));
    }
    Ok(())
}

/// Stop the listener, preserving popup sessions already using open sockets.
/// No running daemon is already the desired state.
pub fn stop() -> io::Result<()> {
    let path = protocol::socket_path();
    let _lock = lock(&path.with_extension("lock"))?;
    match shutdown(&path) {
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused
            ) =>
        {
            Ok(())
        }
        result => result,
    }
}

/// Reuse a running daemon or start one under the startup lock.
/// Installation updates explicitly stop the old daemon before this is called.
pub fn ensure() -> io::Result<String> {
    let path = protocol::socket_path();
    if let Some(reply) = probe(&path)? {
        return Ok(reply);
    }
    let _lock = lock(&path.with_extension("lock"))?;
    if let Some(reply) = probe(&path)? {
        return Ok(reply);
    }

    let mut child = Command::new(std::env::current_exe()?)
        .args(["daemon", "start"])
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let deadline = Instant::now() + WAIT_LIMIT;
    let result = (|| loop {
        if let Some(reply) = probe(&path)? {
            return Ok(reply);
        }
        if let Some(status) = child.try_wait()? {
            return Err(io::Error::other(format!(
                "daemon exited during startup: {status}"
            )));
        }
        if Instant::now() >= deadline {
            return Err(timed_out());
        }
        thread::sleep(Duration::from_millis(2));
    })();
    if result.is_err() {
        // This process owns the child; a failed startup must not leave it
        // racing the next initialization after releasing our lock.
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}
