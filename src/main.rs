use codec::Codec;
use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::Decoder;

mod cmd;
mod codec;
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
    while let Some(Ok(_input)) = stream.next().await {
        sink.send("hey").await.unwrap();
    }
}
