#[cfg(target_os = "windows")]
#[path = ""]
mod m {
    mod serial_windows;
    pub type PortKey = u8;
    pub type Port = serial_windows::ComPort;
}

#[cfg(target_os = "linux")]
#[path = ""]
mod m {
    mod serial_linux;
    pub type PortKey = String;
    pub type Port = serial_linux::TTYPort;
}

pub use m::*;

#[derive(Debug)]
pub struct SerialId {
    pub key: PortKey,
    pub comment: String,
}

pub trait SerialPort: Sized {
    fn list() -> Vec<SerialId>;
    fn open(path: &PortKey, baud: u32, timeout: u32) -> Result<Self, String>;
    fn read(&self, buffer: &mut [u8]) -> Option<usize>;
    fn write(&self, buffer: &[u8]) -> Option<usize>;
}
