use std::sync::Arc;

use anyhow::Result;
use bytes::Bytes;
use tokio::sync::Mutex;

use crate::{codec::Data, connection::Connection, queue::Queue};

pub async fn put(
    connection: &mut Connection,
    queue: Arc<Mutex<Queue>>,
    pri: u32,
    delay: u32,
    ttr: u32,
    data: Bytes,
) -> Result<Vec<Data>> {
    let mut queue = queue.lock().await;
    let id = if delay > 0 {
        queue
            .new_delayed_job(connection.tube().to_string(), ttr, pri, delay, data)
            .await
    } else {
        queue.new_job(connection.tube().to_string(), ttr, pri, data)
    };
    Ok(vec![Data::String("INSERTED".into()), Data::Integer(id)])
}
