use tokio::time::{sleep, Duration};
use std::time::Instant;

#[tokio::main]
async fn main() {
    tokio::spawn(async {
        let start = Instant::now();
        sleep(Duration::from_secs(2)).await;
        println!("Finished async task. Time taken {:?}", start.elapsed());
    });
    let start = Instant::now();
    sleep(Duration::from_secs(4)).await;
    println!("Finished sync task. Time taken {:?}", start.elapsed());
}
