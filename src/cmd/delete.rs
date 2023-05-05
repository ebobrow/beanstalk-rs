use std::sync::{Arc, Mutex};

use anyhow::Result;

use crate::{codec::Data, queue::Queue};

pub fn delete(queue: Arc<Mutex<Queue>>, id: u32) -> Result<Vec<Data>> {
    let mut queue = queue.lock().unwrap();
    if queue.delete_job(id) {
        Ok(vec![Data::String("DELETED".into())])
    } else {
        Ok(vec![Data::String("NOT_FOUND".into())])
    }
}
