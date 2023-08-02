use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        println!("in loop");
        // let (socket, _) = listener.accept().await.unwrap();
        process().await;
    }
    println!("at end");
}

async fn process() {
  println!("in process");
}
