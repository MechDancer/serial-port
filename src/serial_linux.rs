use nix::{
    errno::Errno,
    fcntl::OFlag,
    sys::{
        stat::Mode,
        termios::{self, ControlFlags, SpecialCharacterIndices::*},
    },
};
use std::os::unix::prelude::RawFd;

pub struct TTYPort(RawFd);

impl Drop for TTYPort {
    fn drop(&mut self) {
        let _ = nix::unistd::close(self.0);
    }
}

impl super::SerialPort for TTYPort {
    fn list() -> Vec<String> {
        match std::fs::read_dir("/dev/serial/by-path") {
            Ok(list) => list
                .filter_map(|f| f.ok())
                // .filter(|f| f.path().is_symlink()) // unstable
                .filter_map(|f| f.path().to_str().and_then(|s| Some(s.to_string())))
                .collect::<Vec<_>>(),
            Err(e) => {
                panic!("failed to list serials: {:?}", e);
            }
        }
    }

    fn open(path: &str) -> Result<Self, String> {
        fn map_errno<T>(method: &str, e: Errno) -> Result<T, String> {
            Err(format!("failed to {}: {:?}", method, e))
        }

        let fd = match nix::fcntl::open(path, OFlag::O_RDWR | OFlag::O_NOCTTY, Mode::empty()) {
            Ok(fd) => TTYPort(fd),
            Err(e) => return map_errno("open", e),
        };

        if nix::fcntl::flock(fd.0, nix::fcntl::FlockArg::LockExclusiveNonblock).is_err() {
            return Err(String::from("failed to lock serial exclusive"));
        }

        let mut tty = match termios::tcgetattr(fd.0) {
            Ok(t) => t,
            Err(e) => return map_errno("tcgetattr", e),
        };
        tty.input_flags.remove(termios::InputFlags::all());
        tty.output_flags.remove(termios::OutputFlags::all());
        tty.control_flags.remove(ControlFlags::all());
        tty.local_flags.remove(termios::LocalFlags::all());

        if let Err(e) = termios::cfsetspeed(&mut tty, termios::BaudRate::B230400) {
            return map_errno("cfsetspeed", e);
        }
        tty.control_flags.insert(ControlFlags::CS8);
        tty.control_flags.insert(ControlFlags::CREAD);
        tty.control_flags.insert(ControlFlags::CLOCAL);
        tty.control_chars[VTIME as usize] = 1;
        tty.control_chars[VMIN as usize] = 0;

        if let Err(e) = termios::tcsetattr(fd.0, termios::SetArg::TCSAFLUSH, &tty) {
            return map_errno("tcsetattr", e);
        }

        Ok(fd)
    }

    fn read(&self, buffer: &mut [u8]) -> Option<usize> {
        nix::unistd::read(self.0, buffer).ok()
    }

    fn write(&self, buffer: &[u8]) -> Option<usize> {
        nix::unistd::write(self.0, buffer).ok()
    }
}
