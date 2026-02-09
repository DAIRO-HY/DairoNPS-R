
mod nps;
mod entity;
use nps::nps_bridge::tcp::tcp_bridge::TCPBridge;

fn main() {
    let sd = TCPBridge{
        Client_Id:123,
        CreateTime:12,
        LastRWTime:131,
    };
    println!("Hello, world!{}",sd.ClientId);
}
