use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Mutex;

use crate::{codec::Data, queue::Queue};

pub async fn delete(queue: Arc<Mutex<Queue>>, id: u32) -> Result<Vec<Data>> {
    let mut queue = queue.lock().await;
    if queue.delete_job(id) {
        Ok(vec![Data::String("DELETED".into())])
    } else {
        Ok(vec![Data::String("NOT_FOUND".into())])
    }
}
