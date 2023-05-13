use std::sync::Arc;

use anyhow::Result;
use bytes::Bytes;
use futures_util::{stream::FuturesUnordered, SinkExt, StreamExt};
use tokio::{
    net::TcpStream,
    select,
    sync::{mpsc, Mutex, Notify},
    time::{sleep, Duration},
};
use tokio_util::codec::Framed;

use crate::{
    cmd::{handle_reserved_job, Cmd},
    codec::{BeanstalkCodec, Data},
    queue::Queue,
};

pub struct Connection {
    tube: String,
    watch: Vec<String>,
    stream: Framed<TcpStream, BeanstalkCodec>,

    reserved_job_tx: mpsc::Sender<ReserveCommand>,
    get_job_tx: mpsc::Sender<Option<(u32, u32, Bytes)>>,
    get_job_rx: mpsc::Receiver<Option<(u32, u32, Bytes)>>,

    shutdown: Notify,
}

impl Connection {
    pub fn new(stream: Framed<TcpStream, BeanstalkCodec>) -> Arc<Mutex<Self>> {
        let (reserved_job_tx, reserved_job_rx) = mpsc::channel(100);
        let (get_job_tx, get_job_rx) = mpsc::channel(100);

        let connection = Arc::new(Mutex::new(Self {
            tube: "default".into(),
            watch: vec!["default".into()],
            stream,
            reserved_job_tx,
            shutdown: Notify::new(),
            get_job_tx,
            get_job_rx,
        }));
        tokio::spawn(watch_reserved_jobs(reserved_job_rx, connection.clone()));
        connection
    }

    pub async fn run(&mut self, queue: Arc<Mutex<Queue>>) {
        loop {
            select! {
                Some(input) = self.stream.next() => {
                    match self.handle_frame(queue.clone(), input).await {
                        Ok(data) => self.send_frame(data).await,
                        Err(e) => self.send_frame(vec![Data::String(e.to_string())]).await,
                    }
                }
                Some(recv) = self.get_job_rx.recv() => handle_reserved_job(self, recv).await,
                _ = self.shutdown.notified() => {
                    break;
                }
            }
        }
    }

    pub async fn send_frame(&mut self, frame: Vec<Data>) {
        if !frame.is_empty() {
            self.stream.send(frame).await.unwrap();
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

    pub async fn add_reserved(&mut self, id: u32, ttr: u32) {
        self.reserved_job_tx
            .send(ReserveCommand::Reserve { id, ttr })
            .await
            .unwrap();
    }

    pub async fn remove_reserved(&mut self, id: u32) {
        self.reserved_job_tx
            .send(ReserveCommand::Remove { id })
            .await
            .unwrap();
    }

    pub fn quit(&mut self) {
        self.shutdown.notify_one();
    }

    pub fn get_job_tx(&self) -> mpsc::Sender<Option<(u32, u32, Bytes)>> {
        self.get_job_tx.clone()
    }
}

#[derive(Debug)]
pub enum ReserveCommand {
    Reserve { id: u32, ttr: u32 },
    Remove { id: u32 },
}

async fn watch_reserved_jobs(
    mut reserved_job_rx: mpsc::Receiver<ReserveCommand>,
    connection: Arc<Mutex<Connection>>,
) {
    let mut jobs = FuturesUnordered::new();
    // TODO: is this bad
    let mut removed = Vec::new();
    loop {
        select! {
            Some(cmd) = reserved_job_rx.recv() => {
                match cmd {
                    ReserveCommand::Reserve { id, ttr } => {
                        jobs.push(tokio::spawn(async move {
                            sleep(Duration::from_secs(ttr as u64 - 1)).await;
                            (id, true)
                        }));
                    },
                    ReserveCommand::Remove { id } => removed.push(id)
                }
            }
            Some(Ok((job, safety_margin))) = jobs.next() => {
                if let Some(i) = removed.iter().find(|id| id == &&job) {
                    removed.remove(*i as usize);
                } else if safety_margin {
                    jobs.push(tokio::spawn(async move {
                            sleep(Duration::from_secs(1)).await;
                            (job, false)
                    }));
                    let mut connection = connection.lock().await;
                    connection.send_frame(vec![Data::String("DEADLINE_SOON".into())]).await;
                } else {
                    // TODO: release
                    removed.push(job);
                }
            }
        }
    }
}
