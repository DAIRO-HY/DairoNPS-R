use tokio::{io, try_join};

mod nps;
mod entity;
mod dao;
mod util;
mod constant;

#[tokio::main]
async fn main() -> io::Result<()> {
    util::security_util::init();
    nps::init();
    nps::nps_client::tcp_client::tcp_client_accept::accept().await.unwrap();

    // let task1 = async{
    //     tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    //     println!("task 1 finished");
    //     tokio::io::Result::Ok(())
    // };
    // let task2 = async{
    //     tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    //     println!("task 2 finished");
    //     tokio::io::Result::Ok(())
    // };
    // // let task3 = async{
    // //     tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    // //     println!("task 3 finished");
    // //     tokio::io::Result::Err(io::Error::new(io::ErrorKind::Other, "task 3 error"))
    // // };
    // tokio::try_join!(task1, task2, task4())?;
    println!("-->FINISH<--");
    Ok(())
}

async fn task4() -> io::Result<()> {
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    println!("task 3 finished");
    tokio::io::Result::Err(io::Error::new(io::ErrorKind::Other, "task 3 error"))
}