use std::sync::{Arc, Mutex};

use anyhow::{Ok, Result};

use crate::{codec::Data, connection::Connection, queue::Queue};

pub fn use_tube(
    connection: &mut Connection,
    queue: Arc<Mutex<Queue>>,
    tube: String,
) -> Result<Vec<Data>> {
    let mut queue = queue.lock().unwrap();
    queue.new_tube(&tube);
    connection.use_tube(&tube);
    Ok(vec![Data::String("USING".into()), Data::String(tube)])
}
