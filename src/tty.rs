use crossterm::terminal;
use std::fs::File;
use std::io;

pub struct TtyGuard {
    pub tty: File,
}

impl TtyGuard {
    pub fn new() -> io::Result<Self> {
        let tty = File::options()
            .read(true)
            .write(true)
            .open("/dev/tty")
            .map_err(|e| io::Error::new(e.kind(), "failed to open /dev/tty: not a terminal?"))?;
        terminal::enable_raw_mode()?;

        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let _ = terminal::disable_raw_mode();
            original_hook(info);
        }));

        Ok(TtyGuard { tty })
    }
}

impl Drop for TtyGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

pub fn open_tty_write() -> io::Result<File> {
    File::options()
        .write(true)
        .open("/dev/tty")
        .map_err(|e| io::Error::new(e.kind(), "failed to open /dev/tty for write"))
}
