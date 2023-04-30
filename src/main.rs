use cmd::Cmd;
use codec::Codec;
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Decoder;

mod cmd;
mod codec;
mod parser;
mod settings;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) {
    let codec = Codec::new();
    let (mut sink, mut stream) = codec.framed(socket).split();
    while let Some(input) = stream.next().await {
        match input {
            Ok(data) => {
                // let _cmd = Cmd::try_from(data);
                // TODO: why does this infinitely loop
                // sink.send("unimplemented".to_string()).await.unwrap();
                println!("hey");
            }
            Err(e) => {
                sink.send(e.to_string()).await.unwrap();
            }
        }
        sink.flush().await.unwrap();
    }
}
