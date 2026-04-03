use std::fs::File;
use std::io;
use std::os::unix::io::RawFd;

pub fn open_tty_write() -> io::Result<File> {
    File::options()
        .write(true)
        .open("/dev/tty")
        .map_err(|e| io::Error::new(e.kind(), "failed to open /dev/tty for write"))
}

/// Open `/dev/tty` for both reading and writing, returning the raw file descriptor.
/// The caller is responsible for closing the fd when done.
pub fn open_tty_rw() -> io::Result<RawFd> {
    let fd = unsafe { libc::open(c"/dev/tty".as_ptr(), libc::O_RDWR | libc::O_CLOEXEC) };
    if fd < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(fd)
    }
}
