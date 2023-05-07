use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Mutex;

use crate::{codec::Data, connection::Connection, queue::Queue};

pub async fn delete(
    connection: &mut Connection,
    queue: Arc<Mutex<Queue>>,
    id: u32,
) -> Result<Vec<Data>> {
    connection.remove_reserved(id).await;
    let mut queue = queue.lock().await;
    if queue.delete_job(id) {
        Ok(vec![Data::String("DELETED".into())])
    } else {
        Ok(vec![Data::String("NOT_FOUND".into())])
    }
}
