use std::sync::{Arc, Mutex};

use cmd::Cmd;
use codec::{Codec, Data};
use futures_util::{SinkExt, StreamExt};
use queue::Queue;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Decoder;

mod cmd;
mod codec;
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
            process(queue, socket).await;
        });
    }
}

async fn process(queue: Arc<Mutex<Queue>>, socket: TcpStream) {
    let codec = Codec::new();
    let (mut sink, mut stream) = codec.framed(socket).split();
    while let Some(input) = stream.next().await {
        match input {
            Ok(data) => {
                // TODO: send these errors
                let cmd = Cmd::try_from(data).unwrap();
                let res = cmd.run(queue.clone()).unwrap();
                sink.send(res).await.unwrap();
            }
            Err(e) => {
                sink.send(vec![Data::String(e.to_string())]).await.unwrap();
            }
        }
        sink.flush().await.unwrap();
    }
}
