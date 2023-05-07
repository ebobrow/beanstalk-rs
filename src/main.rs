use std::sync::Arc;

use codec::BeanstalkCodec;
use connection::Connection;

use queue::Queue;
use tokio::{
    net::TcpListener,
    sync::{mpsc, Mutex},
};
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
    let (ready_job_tx, ready_job_rx) = mpsc::channel(100);
    let queue = Arc::new(Mutex::new(Queue::new(ready_job_tx)));

    watch_delay_jobs(queue.clone(), ready_job_rx);

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

fn watch_delay_jobs(queue: Arc<Mutex<Queue>>, mut ready_job_rx: mpsc::Receiver<(String, u32)>) {
    tokio::spawn(async move {
        if let Some((tube, id)) = ready_job_rx.recv().await {
            let mut queue = queue.lock().await;
            queue.queue_job(tube, id);
        }
    });
}
