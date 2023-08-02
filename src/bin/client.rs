
use bytes::Bytes;
use tokio::sync::{mpsc, oneshot};
use mini_redis::client;

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: oneshot::Sender<Option<Bytes>>
    },
    Set {
        key: String,
        value: Bytes,
        resp: oneshot::Sender<()>
    }
}

#[tokio::main]
async fn main() {
    // Capacity of 32 messages
    let (tx, mut rx) = mpsc::channel::<Command>(100);
    let tx1 = tx.clone();

    let t1 = tokio::spawn(async move {
        // Send message to set in redis
        let (respond_tx, respond_rx) = oneshot::channel::<()>();
        tx.send(Command::Set { key: "foo".into(), value: Bytes::from("bar"), resp: respond_tx }).await.unwrap();

        // Await response from oneshot channel
        let res =  respond_rx.await.unwrap();
        println!("Set response {:?}", res);
    });
    let t2 = tokio::spawn(async move {
        let (respond_tx, respond_rs) = oneshot::channel::<Option<Bytes>>();
        // Send message to read value
        tx1.send(Command::Get { key: "foo".into(), resp: respond_tx }).await.unwrap();

        let res = respond_rs.await.unwrap();
        println!("Get response {:?}", res);
    });

    let manager = tokio::spawn(async move {
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        // Receive messages
        while let Some(message) = rx.recv().await {
            match message {
                Command::Get { key, resp } => {
                    // Use oneshot channel to return message to get task
                    let result = client.get(&key).await.unwrap();

                    // Oneshot channel send() doesn't need an await
                    resp.send(result).unwrap();
                },
                Command::Set { key, value, resp } => {
                    let result = client.set(&key, value).await.unwrap();
                    resp.send(result).unwrap();
                }
            }
        }
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();

}