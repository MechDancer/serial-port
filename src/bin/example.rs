use serial_port::{Port, SerialPort};

fn main() {
    use std::{sync::Arc, thread};
    let port = Port::list().into_iter().next().expect("no serial port");
    let port = Arc::new(Port::open(&port.key, 115200, u32::MAX).unwrap());
    {
        let port = port.clone();
        thread::spawn(move || {
            let mut line = String::new();
            while std::io::stdin().read_line(&mut line).is_ok() {
                port.write(line.as_bytes());
                line.clear();
            }
        });
    }
    let mut buf = [0u8; 1024];
    while let Some(n) = port.read(&mut buf) {
        println!("{}", std::str::from_utf8(&buf[..n]).unwrap());
    }
}
