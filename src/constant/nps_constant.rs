// WEB管理端口
pub static WEB_PORT: u16 = 1780;

// 服务端监听TCP端口,客户端通过此端口进行连接
pub static TCP_PORT: &str = "1781";

// 服务端监听UDP端口,客户端通过此端口进行连接
pub static UDP_PORT: &str = "1782";

// 关闭UDP连接池标记
pub const UDP_POOL_CLOSE_FLAG: &[u8; 17] = b":[NPS-POOL-CLOSE]";

// 关闭UDP桥接标记
pub const UDP_BRIDIGE_CLOSE_FLAG: &[u8; 19] = b":[NPS-BRIDGE-CLOSE]";

// 数据统计时间间隔（秒）
pub const STATISTICS_DATA_SIZE_TIMER: u64 = 60;

/**
 * 因为UDP的不确定性,服务端无法检测存活状态,所以
 * 每个一段时间去检测过期的连接
 */
//const val RECYLE_UDP_TIME = 1 * 10 * 1000L
pub const RECYLE_UDP_TIME: u64 = 1 * 60 * 1000;

// 每隔一段时间回收长时间不用的连接池（毫秒）
pub const RECYLE_POOL_TIME_OUT: u64 = 3 * 60 * 1000;

//const RECYLE_POOL_TIME_OUT = 5 * 1000

// UDP桥接连接会话超时(毫秒)
pub const UDP_BRIDGE_TIMEOUT: u64 = 1 * 60 * 1000;

//const UDP_BRIDGE_TIMEOUT = 5 * 1000

/**
 * 心跳间隔时间
 */
pub const HEART_TIME: u64 = 3000;

/**
 * 读取数据缓存大小
 */
pub const READ_UDP_CACHE_SIZE: u64 = 1500;

/**
 * 读取数据缓存大小
 */
pub const READ_CACHE_SIZE: u64 = 32 * 1024;

/**
 * 连接池最大数量
 */
pub const MAX_POOL_COUNT: usize = 6;

/**
 * 连接池最低数量
 * 连接池中的Socket在一段时间内无任何操作
 */
pub const MIN_POOL_COUNT: u64 = 1;

/**
 * 连接池不足时,一次性创建连接数
 */
pub const ADD_POOL_COUNT: u64 = 3;

/**
 * 系统配置
 */
//var systemConfig = SystemConfigDao.SelectOne()

// 管理员用户名
pub static LOGIN_NAME: &str = "admin";

// 管理员登录密码 默认随机6位数
// pub static  LOGIN_PWD:&str = strconv.Itoa(rand.IntN(900000) + 100000);

// 是否开发模式
pub static IS_DEV: bool = false;
