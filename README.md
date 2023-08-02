# Tokio tutorial

## Simple hello world

- Use `await` to write sync-like code
- Futures yield nothing by themselves. They must be awaited to yield.

```rs
use mini_redis::{client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = client::connect("127.0.0.1:6379").await?;

    client.set("hello", "world".into()).await?;

    let result = client.get("hello").await?;
    println!("result {:?}", result);

    // Must await to obtain result from a future
    let zero_result = zero().await;
    Ok(())
}

async fn zero() ->u8 {
    0
}
```

## Socket listener

- Move the hello-world example to `examples/hello-redis.rs`. We can run this by calling `cargo run --example hello-redis`
- Stop the `mini-redis-server`. We will create a custom server that will listen to port `127.0.0.1:6379`

## Blocking example

- Processes each message one by one.
- We don't fall into an infinite loop printing `in loop`. `listener.accept().await` waits for new connections, only then is rest of the code executed.

```rs
use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    // Listen to 127.0.0.1:6379
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    // This loop is awaiting for a socket message. "in loop" is not printed again unless a new message is received
    loop {
        println!("in loop");
        let (socket, _) = listener.accept().await.unwrap();
        process(socket).await;
    }
}

async fn process(socket: TcpStream) {
    let mut connection = Connection::new(socket);

    if let Some(frame) = connection.read_frame().await.unwrap() {
        // Hello world message sent by `hello-redis` client
        println!("got frame {:?}", frame);

        // Respond to `hello-redis`
        let response_frame = Frame::Error("unimplemented".to_string());
        connection.write_frame(&response_frame).await.unwrap();
    }
}
```

- tokio provides async functions for std library utils. A blocking equivalent could be written using `std::net::TcpListener` and `std::net:TcpStream`

## Non-blocking

- Spawn a task for each socket
- Instead of calling `process(socket).await` directly we wrap it in `tokio::spawn()`. This cre

```rs
    loop {
        println!("in loop");
        let (socket, _) = listener.accept().await.unwrap();

        // Process sockets concurrently without blocking
        tokio::spawn(async move {
            process(socket).await;
        });
    }
```

### Tokio tasks

- Tasks are also called asynchronous green threads. They are more lightweight than OS level threads. We can create thousands of tasks in an app.

### Futures vs handles

- Awaiting a handle immediately after spawing a task is no different than directly awaiting the future.
- Await is not immediately called after spawing a task, because that blocks the current task.
- Instead we spawn a task, run some other computation and call await when the result is needed.
- Tasks run on futures. They immediately start executing the future without blocking rest of the code.

```rs
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
```

- The output of the above code is

```
Finished async task. Time taken 2.001438416s
Finished sync task. Time taken 4.002211526s
```

- The spawned task continued working and yielded before the main task even though we blocked the main task.

### `'static` bound, i.e. lifetime

- Tasks have a type of lifetime `'static`. We can't use references to outside data.
- Use `async move` to move async data to the task.
- We must use a synchronization primitive(Arc and Mutex) if data is shared between different tasks.

```rs
// cargo run --example future-vs-task
use tokio::task;

#[tokio::main]
async fn main() {
    let v = vec![1, 2, 3];

    // error- can't use v. To use v, rename the block `async` with `async move`
    task::spawn(async {
        println!("Here's a vec: {:?}", v);
    });
}

```

### `Send` bound

- Tasks spawned must implement `Send`. This allows tasks to be moved between threads while they are suspended by an await.

```rs
use std::rc::Rc;

use tokio::task::{self, yield_now};

#[tokio::main]
async fn main() {
    let handle = tokio::task::spawn(async {
        // Gives error- future cannot be sent between threads safely
        // the trait `Send` is not implemented for `Rc<&str>
        // let rc = Rc::new("hello");
        // println!("{}", rc);

        // Wrap in a block to drop rc after scope
        {
            let rc = Rc::new("hello");
            println!("{}", rc);
        }

        // yield_now() temporarily returns control to the scheduler so that other tasks can be run.
        // The remaining task is executed later
        yield_now().await;

        // This is not printed unless we call handle.await
        println!("yielded");
    });
}
```

## Terminology

## Ownership

Ownership system ensures memory safety in Rust. In Rust **every value has a single owner** and this owner is responsible for managing the value's memory.

```rs
fn main() {
    // s1 is the owner of the string
    let s1 = String::from("hello");

    // Ownership is transferred to s2
    let s2 = s1;
    println!("{}", s2);

    // s1 is unusable now. Attempting to use s1 will result in a compile-time error
    // println!("{}", s2);

    // When s2 goes out of scope, memory for the string is deallocated automatically
} // Memory for s2 deallocated here
```

Ownership rules
- Scope: When a variable comes into scope, it owns its value. When it goes out of scope the value is dropped and memory deallocated.
- Ownership transfer: Ownership can be transferred by using `moves`. The original variable is no longer valid. Only the new variable can use the value.
- Borrowing: Allows a variable to have temporary access to a variable without taking ownership.
- Copy types: Simple types like numbers and boolean implement the `Copy` trait, which allows them to be copied instead of moved.
- Ownership in functions: When values are passed to functions, they are either moved or borrowed depending on function signature.

```rs
fn main() {
    let x = String::from("gg");
    print_string(x);

    // Error- x has been moved
    // println!("afterwards {x}");
}

fn print_string(x: String) {
    println!("{x}");
}
```

### Memory managent in C++

Memory allocation and de-allocation must be manually managed in C++.
  - Call `delete s2` at the end to deallocate memory. Otherwise we have a memory leak. The program continues to operate but the memory used by s2 is not freed.
  - s1 must be set to nullptr after transferring ownership. Otherwise we run into the **dangling pointer problem** where a pointer points to memory that may have been deallocated.

```cpp
#include <iostream>
#include <string>

int main() {
    // "hello" string is created on the heap and owned by the pointer "s1"
    std::string* s1 = new std::string("hello");

    // Ownership is transferred from "s1" to "s2"
    std::string* s2 = s1;
    s1 = nullptr; // Clear the pointer to avoid double deletion

    // Print the value of "s2"
    std::cout << *s2 << std::endl;

    // Attempting to use "s1" will result in undefined behavior or a crash
    // std::cout << *s1 << std::endl; // Undefined behavior or crash

    // Deallocate the memory for the string "hello" manually
    delete s2;

    return 0;
} // Memory for "s2" is deallocated here
```

### Handle

A reference to a value that provides access to some shared state

### Reference-counted

The standard library provides `Rc<T>` which stands for reference-counted. This is a memory management technique to allow **multiple ownership** of data in different parts of the code.

- An `Rc<T>` type allows you to create **smart references**, i.e. lightweight handles to the data.
- `Rc` keeps track of the number of references in a **reference count** variable. When the last reference is dropped, this becomes 0 and the data is automatically deallocated.

## Sharing state

Tokio has two methods of sharing state
1. Guard the shared state with mutex: for simple updates
2. Spawn a task with the state and use message passing to operate on it: Use this for async operations

### 1. Mutex


- Import `bytes::Bytes` using `cargo install bytes`. `Bytes` is roughly a `Arc<Vec<u8>>`. `Bytes` allows shadow cloning, i.e. `clone()` doesn't copy the underlying data. Instead Bytes is a reference-counted handle to some underlying data.
- Wrap the HashMap into `Arc` and `Mutex`
  - Arc: `Atomically Reference Counted`. This is a thread safe reference-counted pointer.

```rs
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
```

- **contention**: The mutex is locked while performing operations in `process()`. Contention here means various threads are trying to obtain a lock on a mutex.
  - When a lock is contented, the thread executing the task must block and wait for the mutex. Since the **whole thread is blocked** other tasks running on the thread get blocked too.
  - `current_thread` runtime flavour: Since only a single thread is used, the mutex is not contended.

#### Dealing with contention

A blocking mutex becomes problematic when contention is high. There are solutions like:
1. Sharding the mutex: Arc contains an array of Mutexes. This way all threads don't contend on the same mutex. `dashmap` crate provides a more production grade map.

    ```rs
    type ShardedDb = Arc<Vec<Mutex<HashMap<String, Vec<u8>>>>>;

    fn new_sharded_db(num_shards: usize) -> ShardedDb {
        let mut db = Vec::with_capacity(num_shards);
        for _ in 0..num_shards {
            db.push(Mutex::new(HashMap::new()));
        }
        Arc::new(db)
    }
    ```

    Write to a shard like this-

    ```rs
    let shard = db[hash(key) % db.len()].lock().unwrap();
    shard.insert(key, value);
    ```

2. Dedicated task to manage state and use message passing.

#### Deadlocks

Deadlock is a situation where multiple async tasks are waiting on each other to complete, resulting in a state where none of the tasks can proceed. They happen when each task is holding onto a resource which the other task needs.

```rs
use std::sync::{Arc, Mutex};
use tokio::task;

async fn task1(data: Arc<Mutex<Vec<u8>>>) {
    // Lock the mutex to access the shared data
    let mut guard = data.lock().await;

    // Do some processing with the shared data

    // Task1 now needs to call task2, but task2 is holding the mutex
    task2(data).await;
}

async fn task2(data: Arc<Mutex<Vec<u8>>>) {
    // Lock the mutex to access the shared data
    let mut guard = data.lock().await;

    // Do some processing with the shared data

    // Task2 now needs to call task1, but task1 is holding the mutex
    task1(data).await;
}

#[tokio::main]
async fn main() {
    let shared_data = Arc::new(Mutex::new(Vec::new()));

    // Spawn two asynchronous tasks
    let task1_handle = task::spawn(task1(Arc::clone(&shared_data)));
    let task2_handle = task::spawn(task2(Arc::clone(&shared_data)));

    // Wait for the tasks to complete
    task1_handle.await.unwrap();
    task2_handle.await.unwrap();
}
```

#### Holding mutex across an `.await`

- A mutex lock must be deallocated before calling `.await`. Mutex doesn't implement `Send`
- This behavior protects us from deadlocks. When `.await` is called the task is suspended and other task which uses the mutex may be picked up **by the same thread**. The new task waits to acquire a lock, which can only happen if the first task can complete.
    - Important: **Mutex is per thread** not per task

```rs
use std::sync::Mutex;

#[tokio::main]
async fn main() {
  let mutex: Mutex<i32> = Mutex::new(0);

  tokio::spawn(async move {
    // Gives error- future cannot be sent between threads safely
    // the trait `Send` is not implemented for `std::sync::MutexGuard<'_, i32>`
    // let mut lock: std::sync::MutexGuard<'_, i32> = mutex.lock().unwrap();
    // *lock += 1;

    // Wrap in a scope so that mutex lock's destructor must run before await
    {
      let mut lock = mutex.lock().unwrap();
      *lock += 1;
    }


    do_something_async().await;
  });
}

async fn do_something_async() {

}
```

We can restructure the code so that `.lock` is not called inside an async function.

```rs
use std::sync::Mutex;

struct CanIncrement {
  mutex: Mutex<i32>
}
impl CanIncrement {
  fn increment(&self) {
    let mut lock = self.mutex.lock().unwrap();
    *lock += 1;
  }
}
#[tokio::main]
async fn main() {
  // Alternate method- only lock mutex in a sync function
  let can_incr = CanIncrement { mutex: Mutex::new(0) };

  tokio::spawn(async move {
    can_incr.increment();

    do_something_async().await;
  });
}

async fn do_something_async() {

}
```

#### Tokio's async mutex

- Tokio's async mutex implements `Send`. It is held across an `await` without issues and can be passed between threads.
- If we replace `std` mutex with tokio mutex in the Send example, our code will compile.
- Tokio mutex provides a way for **multiple tasks running on the same thread** to access shared data safely.
- When a synchronous mutex tries to acquire a lock while it is already locked, it will block execution on that thread. If an async mutex tries to acquire a lock while it is already locked, it will yield execution to the executor
- Use `std::sync::Mutex` when **blocking is acceptable**, i.e. it is okay for the current task to wait
- In async programming, **suspend tasks, not block threads**. When a task is suspended, another task can be performed on it's thread.
- Use synchronous Mutex in async context only if contention is low and the task is not expected to block, like in our example.
- Case when sync mutex is to be avoided
  - 2 processes try to access `mutex` and we have 2 threads available. Mutex writes are instant but each task takes 10 seconds
  - Task 1 on thread A acquires lock and updates mutex. There are 10 seconds for task to elapse. Until then thread A still holds the mutex.
  - Task 2 fires on thread B. When `lock()` is encountered, task 2 must wait 10 seconds to obtain the lock.
  - If this were a sync mutex, thread B is blocked. If this is a tokio mutex, task 2 gets suspended allowing thread B to be used by another task (task 3, 4, 5). Note that there's no performance benefit to task 1 and task 2.

```rs
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
  let mutex: Mutex<i32> = Mutex::new(0);

  tokio::spawn(async move {
    // call await instead of unwrap() on lock
    let mut lock = mutex.lock().await;
    *lock += 1;

    do_something_async().await;
  });
}

async fn do_something_async() {

}
```

## Channels

### Intro

New setup- `main.rs` moved to bin/server.rs. Created a new bin/client.rs. Execute them as `cargo run --bin server`

- The below code fails because `client` can outlive the current function.

```rs
use mini_redis::client;

#[tokio::main]
async fn main() {
  let mut client = client::connect("127.0.0.1:6379").await.unwrap();

  // compile error- async block may outlive the current function,
  // but it borrows `client`, which is owned by the current function
  let set_task = tokio::spawn(async {
    client.set("foo", "bar".into()).await.unwrap();
  });

  let get_task = tokio::spawn(async {
    let res = client.get("foo").await.unwrap();
  });

  set_task.await.unwrap();
  get_task.await.unwrap();
}
```

- Wrap client into a mutex: We can't use await on an std Mutex guard. Solution- use tokio mutex. But since the client is moved in `set_task`, we need to create a second connection.

```rs
  let client = Arc::new(Mutex::new(client::connect("127.0.0.1:6379").await.unwrap()));
  let set_task = tokio::spawn(async move {
    let moved_client = client;
    // future cannot be sent between threads safely
    // the trait `Send` is not implemented for `std::sync::MutexGuard<'_, Client>`
    let mut client = moved_client.lock().unwrap();

    // We could've used std mutex if set operation was local and synchronous
    client.set("foo", "bar".into()).await.unwrap();
  });
```

### Channel types

We can use a single connection to send messages from multiple tasks using channels. We create a dedicated task which handles the connections, then pass messages to this task. Tokio provides a number of channel primitives like:

1. mpsc: Multiple producer single consumer
2. oneshot: Single producer single consumer. A single value can be sent
3. broadcast: Multi-producer multi-consumer. Each receiver sees each value.
4. broadcast: single producer, multi-consumer. Many values are sent but no history is kept. Receivers see the most recent value.

### MPSC (multi producer single consumer)

```rs

use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    // Capacity of 32 messages
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Create multiple senders by cloning tx
    let tx2 = tx.clone();
    tokio::spawn(async move {
        tx.send("gg".to_string()).await.unwrap();
    });

    tokio::spawn(async move {
        tx2.send("wp".to_string()).await.unwrap();
    });

    while let Some(message) = rx.recv().await {
        println!("got {message}");
    }

}
```

- Channel has a capacity of 32 strings. If this is exceeded, awaiting `send` puts the task to sleep until capacity is freed.
- Clone `tx` to create more senders. They can be used in tasks.
- `rx` can't be cloned. It can be used only in one place.
- When all senders (`tx`) go out of scope, the channel is closed. Calling `rx.recv()` will return None.

### Oneshot channel

We use oneshot channels to pass the result back to the tasks.
- Writes to oneshot channels don't need an `await`. They succeed or fail immediately without waiting

```rs

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
```