use serial_port::{Port, PortKey, SerialPort};

fn main() {
    let path = String::from("/dev/ttyUSB0");
    let port = Port::open(&path, 115200, u32::MAX).unwrap();
    let mut buf = [0u8; 256];
    while let Some(n) = port.read(&mut buf) {
        println!("{:?}", std::str::from_utf8(&buf[..n]));
    }
}
