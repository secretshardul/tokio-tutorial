use std::{collections::HashMap, sync::{Arc, Mutex}};

use mini_redis::{Connection, Frame, Command};
use tokio::net::{TcpListener, TcpStream};
use bytes::Bytes;

#[tokio::main]
async fn main() {
    let db = Arc::new(Mutex::new(HashMap::<String, Bytes>::new()));

    // Listen to 127.0.0.1:6379
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    // This loop is awaiting for a socket message. "in loop" is not printed again unless a new message is received
    loop {
        println!("in loop");
        let (socket, _) = listener.accept().await.unwrap();

        // Important: we're cloning the handle, not the hashmap
        let db = db.clone();
        // Process sockets concurrently without blocking
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream,  db: Arc<Mutex<HashMap<String, Bytes>>>) {
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        println!("got frame {:?}", frame);

        let response = match Command::from_frame(frame).unwrap() {
            mini_redis::Command::Set(msg) => {
                // The Arc must be locked to perform writes
                let mut db = db.lock().unwrap();

                db.insert(msg.key().to_string(), msg.value().clone());
                Frame::Simple("OK".to_string())
            },
            mini_redis::Command::Get(msg) => {
                // Lock to read
                let db = db.lock().unwrap();
                if let Some(value) = db.get(msg.key()) {
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            },

            cmd => panic!("unimplemented {:?}", cmd)
        };

        connection.write_frame(&response).await.unwrap();
    }
}
