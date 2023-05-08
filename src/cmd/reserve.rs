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
    let mut queue = queue.lock().await;
    let poll = queue.reserve_job(connection.get_watched_tubes());
    // TODO: this blocks which defeats the whole purpose (does this have to be ANOTHER thread?)
    // and then write tests
    if let Ok(job) = timeout(Duration::from_secs(seconds as u64), poll_fn(|_| poll)).await {
        connection.add_reserved(job.id, job.ttr).await;
        Ok(vec![
            Data::String("RESERVED".into()),
            Data::Integer(job.id),
            Data::Integer(job.data.len() as u32),
            Data::Crlf,
            Data::Bytes(job.data.clone()),
        ])
    } else {
        Ok(vec![Data::String("TIMED_OUT".into())])
    }
}
