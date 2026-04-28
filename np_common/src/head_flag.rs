
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

/**
 * 准备连接到目标服务器
 */
pub const CONNECT_TO_TARGET_SERVER: u8 = 8;

/**
 * 连接池已满标记
 */
pub const POOL_IS_FULL: u8 = 9;
