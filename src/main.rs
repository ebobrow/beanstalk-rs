use std::sync::{Arc, Mutex};

use codec::BeanstalkCodec;
use connection::Connection;

use queue::Queue;
use tokio::net::TcpListener;
use tokio_util::codec::Decoder;

mod cmd;
mod codec;
mod connection;
mod parser;
mod queue;
mod settings;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    let queue = Arc::new(Mutex::new(Queue::new()));

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let queue = queue.clone();

        tokio::spawn(async move {
            let codec = BeanstalkCodec::new();
            let mut connection = Connection::new(codec.framed(socket));
            connection.run(queue).await;
        });
    }
}
