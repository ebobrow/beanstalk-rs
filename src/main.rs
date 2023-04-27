use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Decoder, LinesCodec};

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
    let codec = LinesCodec::new();
    let (mut sink, mut stream) = codec.framed(socket).split();
    while let Some(Ok(input)) = stream.next().await {
        sink.send(input).await.unwrap();
    }
}
