//! Startup-only daemon replacement. Interactive requests never run this path.
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::LazyLock;
use std::thread;
use std::time::{Duration, Instant};

use crate::protocol;

pub const BUILD_ID: &str = env!("ZACRS_BUILD_ID");
const TIMEOUT: Duration = Duration::from_secs(1);
const WAIT_LIMIT: Duration = Duration::from_secs(3);

/// Capture before listening: resolving the executable after replacement could
/// describe the new file rather than the one this process actually loaded.
pub fn executable_id() -> &'static str {
    static ID: LazyLock<String> = LazyLock::new(|| {
        std::env::current_exe()
            .and_then(fs::metadata)
            .map(|metadata| {
                format!(
                    "{}:{}:{}:{}:{}",
                    metadata.dev(),
                    metadata.ino(),
                    metadata.len(),
                    metadata.mtime(),
                    metadata.ctime()
                )
            })
            .unwrap_or_default()
    });
    &ID
}

fn timed_out() -> io::Error {
    io::Error::new(io::ErrorKind::TimedOut, "timed out ensuring current daemon")
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

fn is_current(reply: &str) -> bool {
    let build_matches = reply
        .split_whitespace()
        .any(|field| field.strip_prefix("build_id=") == Some(BUILD_ID));
    build_matches
        && !executable_id().is_empty()
        && reply
            .split_whitespace()
            .any(|field| field.strip_prefix("executable_id=") == Some(executable_id()))
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

/// Installation hook: stop an obsolete daemon without starting a new one in
/// the installer's environment. A matching or absent daemon is left alone.
pub fn retire() -> io::Result<()> {
    let path = protocol::socket_path();
    let _lock = lock(&path.with_extension("lock"))?;
    if let Some(reply) = probe(&path)?
        && !is_current(&reply)
    {
        shutdown(&path)?;
    }
    Ok(())
}

/// Ensure the executable invoking this function owns the listening socket.
/// Return its shell handshake so initialization needs no second socket roundtrip.
pub fn ensure() -> io::Result<String> {
    let path = protocol::socket_path();
    let _lock = lock(&path.with_extension("lock"))?;
    if let Some(reply) = probe(&path)? {
        if is_current(&reply) {
            return Ok(reply);
        }
        shutdown(&path)?;
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
            if is_current(&reply) {
                return Ok(reply);
            }
            return Err(io::Error::other("another daemon owns the socket"));
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
