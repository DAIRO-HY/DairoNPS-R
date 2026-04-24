mod npc_error;
mod bridge;
mod pool;
mod header_util;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    println!("hello npc");
    Ok(())
}