use std::sync::Arc;

use anyhow::Result;
use bytes::Bytes;
use tokio::sync::Mutex;

use crate::{codec::Data, queue::Queue};

pub async fn list_tubes(queue: Arc<Mutex<Queue>>) -> Result<Vec<Data>> {
    let queue = queue.lock().await;
    let body = format!(
        "---\n{}",
        queue
            .tube_names()
            .map(|name| format!("- {name}\n"))
            .collect::<String>()
    );
    Ok(vec![
        Data::String("OK".into()),
        Data::Integer(body.len() as u32),
        Data::Crlf,
        Data::Bytes(Bytes::copy_from_slice(body.as_bytes())),
    ])
}
