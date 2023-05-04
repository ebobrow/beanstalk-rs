use std::sync::{Arc, Mutex};

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use crate::{
    cmd::Cmd,
    codec::{BeanstalkCodec, Data},
    queue::Queue,
};

pub struct Connection {
    tube: String,
    watch: Vec<String>,
    reserved: Vec<u32>,
    stream: Framed<TcpStream, BeanstalkCodec>,
}

impl Connection {
    pub fn new(stream: Framed<TcpStream, BeanstalkCodec>) -> Self {
        Self {
            tube: "default".into(),
            watch: vec!["default".into()],
            reserved: Vec::new(),
            stream,
        }
    }

    pub async fn run(&mut self, queue: Arc<Mutex<Queue>>) {
        while let Some(input) = self.stream.next().await {
            match self.handle_frame(queue.clone(), input) {
                Ok(data) => self.stream.send(data).await.unwrap(),
                Err(e) => self
                    .stream
                    .send(vec![Data::String(e.to_string())])
                    .await
                    .unwrap(),
            }
            self.stream.flush().await.unwrap();
        }
    }

    pub fn handle_frame(
        &mut self,
        queue: Arc<Mutex<Queue>>,
        frame: Result<Vec<Data>>,
    ) -> Result<Vec<Data>> {
        let cmd = Cmd::try_from(frame?)?;
        cmd.run(queue)
    }
}
