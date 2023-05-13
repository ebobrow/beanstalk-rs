use std::sync::Arc;

use anyhow::Result;
use bytes::Bytes;
use tokio::{
    select,
    sync::Mutex,
    time::{sleep, Duration},
};

use crate::{
    codec::Data,
    connection::Connection,
    queue::{Job, Queue},
};

pub async fn reserve_with_timeout(
    connection: &mut Connection,
    queue: Arc<Mutex<Queue>>,
    seconds: u32,
) -> Result<Vec<Data>> {
    let get_job_tx = connection.get_job_tx();
    let watched_tubes = connection.get_watched_tubes().to_vec();
    // TODO: tests
    tokio::spawn(async move {
        let timer = sleep(Duration::from_secs(seconds as u64));
        tokio::pin!(timer);
        loop {
            let try_reserve = try_reserve(queue.clone(), watched_tubes.clone());
            tokio::pin!(try_reserve);
            select! {
                _ = &mut timer => {
                    get_job_tx.send(None).await.unwrap();
                    break;
                }
                res = &mut try_reserve => {
                    if let Some(job) = res {
                        get_job_tx
                            .send(Some((job.id, job.ttr, job.data.clone())))
                            .await
                            .unwrap();
                        break;
                    } else {
                        continue;
                    }
                }
            }
        }
    });
    Ok(Vec::new())
}

async fn try_reserve(queue: Arc<Mutex<Queue>>, watch_list: Vec<String>) -> Option<Job> {
    let mut queue = queue.lock().await;
    queue.reserve_job(watch_list).cloned()
}

pub async fn handle_reserved_job(connection: &mut Connection, job: Option<(u32, u32, Bytes)>) {
    if let Some((id, ttr, data)) = job {
        connection.add_reserved(id, ttr).await;
        connection
            .send_frame(vec![
                Data::String("RESERVED".into()),
                Data::Integer(id),
                Data::Integer(data.len() as u32),
                Data::Crlf,
                Data::Bytes(data.clone()),
            ])
            .await;
    } else {
        connection
            .send_frame(vec![Data::String("TIMED_OUT".into())])
            .await;
    }
}
