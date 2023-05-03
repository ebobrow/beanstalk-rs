use std::sync::{Arc, Mutex};

use anyhow::Result;
use bytes::Bytes;

use crate::{codec::Data, queue::Queue};

pub fn put(
    queue: Arc<Mutex<Queue>>,
    pri: u32,
    delay: u32,
    ttr: u32,
    data: Bytes,
) -> Result<Vec<Data>> {
    let mut queue = queue.lock().unwrap();
    let id = queue.new_job("default".into(), ttr, pri, data);
    Ok(vec![Data::String("INSERTED".into()), Data::Integer(id)])
}
