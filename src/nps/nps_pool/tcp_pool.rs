use tokio::net::TcpStream;

// package tcp_pool
// 
// import "net"
// 
// TCP连接池
pub struct TCPPool {
	pub tcp: TcpStream,

	// 创建时间(秒)
	pub create_time:u64,
}
