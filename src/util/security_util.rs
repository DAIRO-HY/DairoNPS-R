// package SecurityUtil
//
// import (
// 	"fmt"
// 	"math/rand"
// )
use rand::seq::SliceRandom;
use std::sync::OnceLock;

// 服务器端加密秘钥
pub static SERVER_SECURITY_KEY: OnceLock<[u8; 256]> = OnceLock::new();

// 客户端加密秘钥
pub static CLIENT_SECURITY_KEY: OnceLock<[u8; 256]> = OnceLock::new();
pub fn init() {
    let mut server_key = std::array::from_fn(|i| i as u8);
    server_key.shuffle(&mut rand::rng());// 打乱数组

    let mut client_key = [0u8; 256];
    for (i, &v) in server_key.iter().enumerate() {
        client_key[v as usize] = i as u8;
    }
    SERVER_SECURITY_KEY.set(server_key).unwrap();
    CLIENT_SECURITY_KEY.set(client_key).unwrap();
}

// 客户端加密秘钥
// pub static ClientSecurityKey:OnceLock<[u8]> = OnceLock::new().get_or_init(|| {
// 	let mut key = [0u8; 256];
// 	for i in 0..256 {
// 		key[i] = i as u8;
// 	}
// 	key
// });

// func init() {
// 	for i := range ServerSecurityKey {
// 		ServerSecurityKey[i] = uint8(i)
// 	}
//
// 	// 打乱数组
// 	rand.Shuffle(256, func(i, j int) {
// 		ServerSecurityKey[i], ServerSecurityKey[j] = ServerSecurityKey[j], ServerSecurityKey[i]
// 	})
// 	//for i, it := range ServerSecurityKey {
// 	//	fmt.Printf("%d->%d\n", i, it)
// 	//}
// 	fmt.Println("----------------------------------------------------------------------------------------------")
//
// 	//服务端数组的值是客户端数组的序号,对应的服务端数组的序号则是客户端数组的值
// 	for i, it := range ServerSecurityKey {
// 		ClientSecurityKey[it] = uint8(i)
// 	}
// 	//for i, it := range ClientSecurityKey {
// 	//	fmt.Printf("%d->%d\n", i, it)
// 	//}
// }
//
// /**
//  * 加密数据
//  * @param data 要加密的数据
//  * @param len 要加密的数据长度
//  */
// fn mapping(data: [u8], len: usize) {
// 	for i in 0..len{
// 		let value = data[i];
// 		data[i] = ServerSecurityKey[value as usize];
// 	}
// 	// for i := 0; i < len; i++ {
// 	// 	value := data[i]
// 	// 	data[i] = ServerSecurityKey[value]
// 	// }
// }
