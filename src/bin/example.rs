use serial_port::{Port, SerialPort};

fn main() {
    let port = Port::open(&4, 115200, u32::MAX).unwrap();
    let mut buf = [0u8; 256];
    while let Some(n) = port.read(&mut buf) {
        println!("{:?}", std::str::from_utf8(&buf[..n]));
    }
}
