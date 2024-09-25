// ./send/src/main.rs
use std::net::{UdpSocket, Ipv4Addr};
use std::thread;
use std::time::Duration;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:8888").unwrap();
    // let multicast_addr = Ipv4Addr::new(234, 2, 2, 2);
    // let inter = Ipv4Addr::new(0,0,0,0);
    // socket.join_multicast_v4(&multicast_addr, &inter).unwrap();

    socket.set_read_timeout(Some(Duration::from_millis(200))).unwrap();
    const COUNT: usize = 100;
    loop {
        let buf = "hello";
        println!("snd: {buf}");
        // socket.send_to(buf.as_bytes(), "234.2.2.2:6999").unwrap();
        socket.send_to(buf.as_bytes(), "224.1.2.3:6999").unwrap();
        // socket.send_to(buf.as_bytes(), "239.255.255.250:6999").unwrap();
        // socket.send_to(buf.as_bytes(), "239.255.255.250:6999").unwrap();
        // socket.send_to(buf.as_bytes(), "239.255.255.250:6999").unwrap();
        // socket.send_to(buf.as_bytes(), "239.255.255.250:6999").unwrap();
        let mut buf = [0u8; 2048];
        if let Ok((s, _)) = socket.recv_from(&mut buf) {
            println!("{}", String::from_utf8_lossy(&buf[..s]));
            break;
        } else {
            continue;
        }
    }
    thread::sleep(std::time::Duration::from_millis(1000));
    // socket.leave_multicast_v4(&multicast_addr, &inter).unwrap();

}
