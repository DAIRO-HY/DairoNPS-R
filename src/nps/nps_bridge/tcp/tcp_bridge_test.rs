//package tcp_bridge

// import (
//     "DairoNPS/constant/NPSConstant"
//     "DairoNPS/dao/dto"
//     "DairoNPS/util/ChannelStatisticsUtil"
//     "DairoNPS/util/LogUtil"
//     "DairoNPS/util/SecurityUtil"
//     "DairoNPS/util/WriterUtil"
//     "fmt"
//     "net"
//     "strconv"
//     "sync/atomic"
//     "time"
// )


use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf};
use tokio::io::WriteHalf;
use tokio::sync::mpsc;
use tokio::{io, try_join};
use crate::entity::channel::Channel;

// type RemoveBridgeFunc func(bridge *TCPBridge)

// TCPBridge TCP桥接信息
pub struct TCPBridge  {

    pub proxy_reader: ReadHalf<TcpStream>,
    pub proxy_writer: WriteHalf<TcpStream>,

    pub client_reader: ReadHalf<TcpStream>,
    pub client_writer: WriteHalf<TcpStream>,

    // 客户端ID
    pub client_id: u64,

    //当前隧道
    pub channel: Channel,
    //
    // // 隧道代理端TCP链接
    // // ProxyTCP net.Conn
    //
    // // 与客户端的TCP链接
    // // ClientTCP net.Conn
    //
    // //代理连接入方向是否被关闭
    // proxyInIsClosed: Boolean,
    //
    // //客户端连接入方向是否被关闭
    // clientInIsClosed: Boolean,

    // 创建时间(毫秒)
    pub create_time: u64,

    // 记录最后通信时间(毫秒)
    pub last_rw_time: u64,

    //隧道流量统计
    // channelDataSize *ChannelStatisticsUtil.ChannelDataSize
}

impl TCPBridge {
    
    /**
    * 开始桥接传输数据
     */
    async fn start(self) -> io::Result<()>{
        // mine.channelDataSize = ChannelStatisticsUtil.Get(mine.Channel.Id)
        //
        // //发送目标端口信息
        // mine.sendHeaderToClient()
        // go mine.receiveByProxySendToClient()
        // mine.receiveByClientSendToProxy()

        let p2c = Self::proxy_to_client(self.proxy_reader,self.client_writer);
        let c2p = Self::client_to_proxy(self.client_reader,self.proxy_writer);
        try_join!(p2c,c2p)?;
        Ok(())
    }

    async fn proxy_to_client(mut proxy_reader: ReadHalf<TcpStream>, mut client_writer: WriteHalf<TcpStream>) -> io::Result<()>{
        let mut buf = [0u8; 4096];
        loop{
            let n = proxy_reader.read(&mut buf).await?;
            if n == 0{
                break;
            }
            client_writer.write_all(&buf[..n]).await?;
        }
        Ok(())
    }

    async fn client_to_proxy(mut client_reader: ReadHalf<TcpStream>, mut proxy_writer: WriteHalf<TcpStream>) -> io::Result<()>{
        let mut buf = [0u8; 4096];
        loop{
            let n = client_reader.read(&mut buf).await?;
            if n == 0{
                break;
            }
            proxy_writer.write_all(&buf[..n]).await?;
        }
        Ok(())
    }


    /**
    * 发送目标端口信息
     */
    fn sendHeaderToClient(&self) {

        // //将加密类型及目标端口 格式:加密状态|端口  1|80   1|127.0.0.1:80
        // //1:加密  0:不加密
        // let header = strconv.Itoa(mine.Channel.SecurityState) + "|" + mine.Channel.TargetPort
        // let headerData = []uint8(header)
        //
        // //写入数据长度标识
        // data: = []uint8{uint8(len(headerData))}
        //
        // //写入数据
        // data = append(data, headerData...)
        // err : = WriterUtil.WriteFull(mine.ClientTCP, data)
        // if err != nil {
        // LogUtil.Error(fmt.Sprintf("往客户端发送头部失败 err:%q", err))
        // mine.ClientTCP.Close()
        // return
        // }
    }

    /**
    * 从代理服务接收数据发送到客户端
     */
    async fn receiveByProxySendToClient(&mut self) {
        let mut buf = [0u8; 1024];
        loop {
            match self.proxy_reader.read(&mut buf).await {
                Ok(0) => {
                    println!("s:连接已关闭");
                    return;
                }
                Ok(n) => {
                    // tx.send(&buf[..n]).await.unwrap();

                    // let received = String::from_utf8_lossy(&buf[..n]);
                    // println!("s:收到客户端消息: {}", received);
                    // tx.send("-->server:我已经收到你的消息".to_string()).await.unwrap();

                    self.client_writer.write_all(&buf[..n]).await.unwrap();
                }
                Err(e) => {
                    eprintln!("s:读取错误: {}", e);
                    return;
                }
            }
        }




        // data := make([]uint8, NPSConstant.READ_CACHE_SIZE)
        // for {
        //     n, readErr := mine.ProxyTCP.Read(data)
        //     if n > 0 {
        //
        //         //记录最后通信时间
        //         mine.LastRWTime = time.Now().UnixMilli()
        //
        //         //原子递增
        //         atomic.AddInt64(&mine.channelDataSize.InData, int64(n))
        //         if mine.Channel.SecurityState == 1 { //加密数据
        //             SecurityUtil.Mapping(data, n)
        //         }
        //
        //         //将读取到的数据立即发送客户端
        //         writeErr := WriterUtil.WriteFull(mine.ClientTCP, data[:n])
        //         if writeErr != nil {
        //             break
        //         }
        //     }
        //     if readErr != nil {
        //         break
        //     }
        // }
        //
        // //关闭客户端的输出流
        // mine.ClientTCP.(*net.TCPConn).CloseWrite()
        //
        // //关闭代理端的输入流
        // mine.ProxyTCP.(*net.TCPConn).CloseRead()
        //
        // //标记代理连接读操作已经关闭
        // mine.proxyInIsClosed = true
        // mine.recycle()
    }
    //
    // // 从客户端接收发送到代理服务器
    // func (mine *TCPBridge) receiveByClientSendToProxy() {
    //     data := make([]uint8, NPSConstant.READ_CACHE_SIZE)
    //     for {
    //         n, readErr := mine.ClientTCP.Read(data)
    //         if n > 0 {
    //
    //             //记录最后通信时间
    //             mine.LastRWTime = time.Now().UnixMilli()
    //
    //             //出网统计 原子递增
    //             atomic.AddInt64(&mine.channelDataSize.OutData, int64(n))
    //             if mine.Channel.SecurityState == 1 { //加密数据
    //                 SecurityUtil.Mapping(data, n)
    //             }
    //
    //             //将读取到的数据立即发送客户端
    //             writeErr := WriterUtil.WriteFull(mine.ProxyTCP, data[:n])
    //             if writeErr != nil {
    //                 break
    //             }
    //         }
    //         if readErr != nil {
    //             break
    //         }
    //     }
    //
    //     //关闭客户端的输出流
    //     mine.ProxyTCP.(*net.TCPConn).CloseWrite()
    //
    //     //关闭代理端的输入流
    //     mine.ClientTCP.(*net.TCPConn).CloseRead()
    //
    //     //标记客户端读操作已经关闭
    //     mine.clientInIsClosed = true
    //     mine.recycle()
    // }
    //
    // /**
    // * 资源回收
    //  */
    // func (mine *TCPBridge) recycle() {
    //     if mine.proxyInIsClosed && mine.clientInIsClosed {
    //         mine.ClientTCP.Close()
    //         mine.ProxyTCP.Close()
    //         removeBridge(mine)
    //     }
    // }
    //
    // /**
    // * 关闭连接
    //  */
    // func (mine *TCPBridge) shutdown() {
    //     mine.ClientTCP.Close()
    //     mine.ProxyTCP.Close()
    // }
}