use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::main]
async fn main() {
    // TODO: this is ugly code
    let mut stream = TcpStream::connect("127.0.0.1:3000").await.unwrap();
    stream.write_all(b"put 1 1 1 1\r\nh\r\n").await.unwrap();

    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer[..]).await.unwrap();
    println!("{:?}", std::str::from_utf8(&buffer[..n]));

    stream
        .write_all(b"reserve-with-timeout 2\r\n")
        .await
        .unwrap();

    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer[..]).await.unwrap();
    println!("{:?}", std::str::from_utf8(&buffer[..n]));

    stream
        .write_all(b"reserve-with-timeout 2\r\n")
        .await
        .unwrap();

    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer[..]).await.unwrap();
    println!("{:?}", std::str::from_utf8(&buffer[..n]));
}
