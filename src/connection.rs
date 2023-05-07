use std::sync::Arc;

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpStream, sync::Mutex};
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
            match self.handle_frame(queue.clone(), input).await {
                Ok(data) => self.stream.send(data).await.unwrap(),
                Err(e) => self
                    .stream
                    .send(vec![Data::String(e.to_string())])
                    .await
                    .unwrap(),
            }
        }
    }

    pub async fn handle_frame(
        &mut self,
        queue: Arc<Mutex<Queue>>,
        frame: Result<Vec<Data>>,
    ) -> Result<Vec<Data>> {
        let cmd = Cmd::try_from(frame?)?;
        cmd.run(self, queue).await
    }

    pub fn use_tube(&mut self, tube: impl ToString) {
        self.tube = tube.to_string();
    }

    pub fn tube(&self) -> &str {
        self.tube.as_ref()
    }

    pub fn get_watched_tubes(&self) -> &[String] {
        &self.watch
    }

    pub fn watch(&mut self, tube: String) {
        if !self.watch.contains(&tube) {
            self.watch.push(tube);
        }
    }

    pub fn ignore(&mut self, tube: String) {
        if let Some((i, _)) = self
            .watch
            .iter()
            .enumerate()
            .find(|(_, name)| name == &&tube)
        {
            self.watch.remove(i);
        }
    }
}
