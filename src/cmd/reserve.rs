use std::{future::poll_fn, sync::Arc};

use anyhow::Result;
use tokio::{
    sync::Mutex,
    time::{timeout, Duration},
};

use crate::{codec::Data, connection::Connection, queue::Queue};

pub async fn reserve_with_timeout(
    connection: &mut Connection,
    queue: Arc<Mutex<Queue>>,
    seconds: u32,
) -> Result<Vec<Data>> {
    let get_job_tx = connection.get_job_tx();
    let watched_tubes = connection.get_watched_tubes().to_vec();
    tokio::spawn(async move {
        let mut queue = queue.lock().await;
        // TODO: tests
        // TODO: this like freezes it in place so that even after a new job gets added it doesn't
        // see it
        //     - Maybe instead have another channel and when a new job gets added in main it
        //       notifies us
        let poll = queue.reserve_job(watched_tubes);
        if let Ok(job) = timeout(Duration::from_secs(seconds as u64), poll_fn(|_| poll)).await {
            get_job_tx
                .send(Some((job.id, job.ttr, job.data.clone())))
                .await
                .unwrap();
        } else {
            get_job_tx.send(None).await.unwrap();
        }
    });
    Ok(Vec::new())
}
