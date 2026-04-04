use std::fs::File;
use std::io;
use std::os::fd::{FromRawFd, OwnedFd};

pub fn open_tty_write() -> io::Result<File> {
    File::options()
        .write(true)
        .open("/dev/tty")
        .map_err(|e| io::Error::new(e.kind(), "failed to open /dev/tty for write"))
}

pub fn open_tty_rw() -> io::Result<OwnedFd> {
    let fd = unsafe { libc::open(c"/dev/tty".as_ptr(), libc::O_RDWR | libc::O_CLOEXEC) };
    if fd < 0 {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "failed to open /dev/tty for read/write",
        ))
    } else {
        Ok(unsafe { OwnedFd::from_raw_fd(fd) })
    }
}
