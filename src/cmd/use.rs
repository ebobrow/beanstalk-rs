use std::sync::Arc;

use anyhow::{Ok, Result};
use tokio::sync::Mutex;

use crate::{codec::Data, connection::Connection, queue::Queue};

pub async fn use_tube(
    connection: &mut Connection,
    queue: Arc<Mutex<Queue>>,
    tube: String,
) -> Result<Vec<Data>> {
    let mut queue = queue.lock().await;
    queue.new_tube(&tube);
    connection.use_tube(&tube);
    Ok(vec![Data::String("USING".into()), Data::String(tube)])
}
