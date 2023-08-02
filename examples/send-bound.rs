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
