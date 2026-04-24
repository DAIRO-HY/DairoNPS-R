mod npc_error;
mod bridge;
mod pool;
mod session;
mod header_util;
mod application;
mod security_util;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    println!("hello npc");
    Ok(())
}