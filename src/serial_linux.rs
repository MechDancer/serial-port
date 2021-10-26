use crate::{PortKey, SerialId, SerialPort};
use nix::{
    errno::Errno,
    fcntl::{self, fcntl, FcntlArg, FdFlag, FlockArg, OFlag},
    libc::FD_CLOEXEC,
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

impl SerialPort for TTYPort {
    fn list() -> Vec<SerialId> {
        match std::fs::read_dir("/dev/serial/by-path") {
            Ok(list) => list
                .filter_map(|f| f.ok())
                // .filter(|f| f.path().is_symlink()) // unstable
                .filter_map(|f| {
                    f.file_name().to_str().and_then(|s| {
                        Some(SerialId {
                            key: s.to_string(),
                            comment: s.to_string(),
                        })
                    })
                })
                .collect::<Vec<_>>(),
            Err(_) => {
                vec![]
            }
        }
    }

    fn open(key: &PortKey, baud: u32, timeout: u32) -> Result<Self, String> {
        fn map_errno<T>(method: &str, e: Errno) -> Result<T, String> {
            Err(format!("failed to {}: {:?}", method, e))
        }

        let fd = match fcntl::open(
            format!("/dev/serial/by-path/{}", key).as_str(),
            OFlag::O_RDWR | OFlag::O_NOCTTY,
            Mode::empty(),
        ) {
            Ok(fd) => TTYPort(fd),
            Err(e) => return map_errno("open", e),
        };

        if fcntl::flock(fd.0, FlockArg::LockExclusiveNonblock).is_err() {
            return Err(String::from("failed to lock serial exclusive"));
        }

        let mut flags = fcntl(fd.0, FcntlArg::F_GETFD).unwrap();
        flags |= FD_CLOEXEC;
        fcntl(fd.0, FcntlArg::F_SETFD(FdFlag::from_bits(flags).unwrap())).unwrap();

        let mut tty = match termios::tcgetattr(fd.0) {
            Ok(t) => t,
            Err(e) => return map_errno("tcgetattr", e),
        };
        tty.input_flags.remove(termios::InputFlags::all());
        tty.output_flags.remove(termios::OutputFlags::all());
        tty.control_flags.remove(ControlFlags::all());
        tty.local_flags.remove(termios::LocalFlags::all());

        if let Err(e) = termios::cfsetspeed(&mut tty, baud_rate_translate(baud)) {
            return map_errno("cfsetspeed", e);
        }
        tty.control_flags.insert(ControlFlags::CS8);
        tty.control_flags.insert(ControlFlags::CREAD);
        tty.control_flags.insert(ControlFlags::CLOCAL);
        tty.control_chars[VTIME as usize] = (timeout / 100) as u8;
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

fn baud_rate_translate(baud: u32) -> termios::BaudRate {
    match baud {
        9600 => termios::BaudRate::B9600,
        115200 => termios::BaudRate::B115200,
        230400 => termios::BaudRate::B230400,
        460800 => termios::BaudRate::B460800,
        _ => panic!("unsupported baud rate: {}", baud),
    }
}
