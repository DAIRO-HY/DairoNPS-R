use tokio::io;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use bytes::{BufMut, Bytes, BytesMut};

/**
 * 客户端与服务器端通信连接标记
 */
pub const CLIENT_TO_SERVER_MAIN_CONNECTION: u8 = 0;

/**
 * 与客户端通信心跳标记
 */
pub const MAIN_HEART_BEAT: u8 = 1;

/**
 * 向客户端发送clientId
 */
pub const SERVER_TO_CLIENT_ID: u8 = 2;

/**
 * 向客户端申请TCP连接池请求
 */
pub const REQUEST_TCP_POOL: u8 = 3;

/**
 * 向客户端申请UDP连接池请求
 */
pub const REQUEST_UDP_POOL: u8 = 4;

/**
 * 服务器向客户端同步当前处于激活状态的UDP连接池端口
 */
pub const SYNC_ACTIVE_POOL_UDP_PORT: u8 = 5;

/**
 * 向客户端同步当前处于激活状态的UDP连接端口
 */
const SYNC_ACTIVE_BRIDGE_UDP_PORT: u8 = 6;

/**
 * 向客户端发送clientId
 */
pub const SECURITY_CLIENT_KEY: u8 = 7;

//关闭标记指令
pub const CLOSE_CMD: &[u8; 13] = b"@->[CLOSE]<-@";

/**
 * 获取客户端Socket头部信息
 */
pub async fn get_header(tcp_stream: &mut TcpStream) -> io::Result<String> {
    //读取一个字节,该字节代表key长度
    let mut header_len_data = [0u8; 1];
    tcp_stream.read_exact(&mut header_len_data).await?;
    // if _, err := io.ReadFull(clientSocket, headerLenData); err != nil {
    // 	return "", err
    // }

    //得到头部部分数据长度
    let header_len = header_len_data[0] as usize;
    let mut header_data = vec![0u8; header_len];
    // if _, err := io.ReadFull(clientSocket, headerData); err != nil {
    // 	return "", err
    // }
    tcp_stream.read_exact(&mut header_data).await?;
    let header = String::from_utf8_lossy(&header_data).into_owned();
    Ok(header)
}

/// 构建发送给客户端的头部数据
pub fn make_header_data(flag: u8, message: &str) -> Bytes {
    let mut bm = BytesMut::with_capacity(message.as_bytes().len() + 2);
    bm.put_u8(flag);
    bm.put_u8(message.len() as u8);
    bm.put_slice(message.as_bytes());
    bm.freeze()
}
