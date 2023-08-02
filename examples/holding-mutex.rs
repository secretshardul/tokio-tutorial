use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
  let mutex: Mutex<i32> = Mutex::new(0);

  tokio::spawn(async move {
    let mut lock = mutex.lock().await;
    *lock += 1;

    do_something_async().await;
  });
}

async fn do_something_async() {

}