pub trait SerialPort: Sized {
    fn list() -> Vec<String>;
    fn open(path: &str, baud: u32, timeout: u32) -> Result<Self, String>;
    fn read(&self, buffer: &mut [u8]) -> Option<usize>;
    fn write(&self, buffer: &[u8]) -> Option<usize>;
}

#[cfg(target_os = "windows")]
mod serial_windows;

#[cfg(target_os = "windows")]
pub type Port = serial_windows::ComPort;

#[cfg(target_os = "linux")]
mod serial_linux;

#[cfg(target_os = "linux")]
pub type Port = serial_linux::TTYPort;
