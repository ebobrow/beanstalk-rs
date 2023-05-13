use std::sync::Arc;

use anyhow::Result;
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
    let watched_tubes = connection.get_watched_tubes().to_vec();
    let timer = sleep(Duration::from_secs(seconds as u64));
    tokio::pin!(timer);
    loop {
        let try_reserve = try_reserve(queue.clone(), watched_tubes.clone());
        tokio::pin!(try_reserve);
        select! {
            _ = &mut timer => {
                return Ok(vec![Data::String("TIMED_OUT".into())]);
            }
            res = &mut try_reserve => {
                if let Some(job) = res {
                    connection.add_reserved(job.id, job.ttr).await;
                    return Ok(vec![
                        Data::String("RESERVED".into()),
                        Data::Integer(job.id),
                        Data::Integer(job.data.len() as u32),
                        Data::Crlf,
                        Data::Bytes(job.data.clone()),
                    ]);
                } else {
                    continue;
                }
            }
        }
    }
}

async fn try_reserve(queue: Arc<Mutex<Queue>>, watch_list: Vec<String>) -> Option<Job> {
    let mut queue = queue.lock().await;
    queue.reserve_job(watch_list).cloned()
}

pub async fn reserve_job(
    connection: &mut Connection,
    queue: Arc<Mutex<Queue>>,
    id: u32,
) -> Result<Vec<Data>> {
    let mut queue = queue.lock().await;
    if let Some(job) = queue.reserve_by_id(id) {
        connection.add_reserved(job.id, job.ttr).await;
        Ok(vec![
            Data::String("RESERVED".into()),
            Data::Integer(job.id),
            Data::Integer(job.data.len() as u32),
            Data::Crlf,
            Data::Bytes(job.data.clone()),
        ])
    } else {
        Ok(vec![Data::String("NOT_FOUND".into())])
    }
}
