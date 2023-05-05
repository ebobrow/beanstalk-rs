use std::sync::{Arc, Mutex};

use anyhow::Result;
use bytes::Bytes;

use crate::{codec::Data, connection::Connection, queue::Queue};

pub fn put(
    connection: &mut Connection,
    queue: Arc<Mutex<Queue>>,
    pri: u32,
    delay: u32,
    ttr: u32,
    data: Bytes,
) -> Result<Vec<Data>> {
    let mut queue = queue.lock().unwrap();
    let id = queue.new_job(connection.tube().to_string(), ttr, pri, data);
    Ok(vec![Data::String("INSERTED".into()), Data::Integer(id)])
}
