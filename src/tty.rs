use std::fs::File;
use std::io;

pub fn open_tty_write() -> io::Result<File> {
    File::options()
        .write(true)
        .open("/dev/tty")
        .map_err(|e| io::Error::new(e.kind(), "failed to open /dev/tty for write"))
}
