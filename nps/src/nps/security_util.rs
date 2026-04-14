use rand::seq::SliceRandom;
use std::sync::LazyLock;

// 服务器端加密秘钥
pub static SERVER_SECURITY_KEY: LazyLock<[u8; 256]> = LazyLock::new(||{
    let mut server_key = std::array::from_fn(|i| i as u8);
    server_key.shuffle(&mut rand::rng());// 打乱数组
    server_key
});

// 客户端加密秘钥
pub static CLIENT_SECURITY_KEY: LazyLock<[u8; 256]> = LazyLock::new(||{
    let mut client_key = [0u8; 256];
    for (i, &v) in SERVER_SECURITY_KEY.iter().enumerate() {
        client_key[v as usize] = i as u8;
    }
    client_key
});
