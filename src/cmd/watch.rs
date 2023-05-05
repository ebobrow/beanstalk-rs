use anyhow::Result;

use crate::{codec::Data, connection::Connection};

pub fn watch(connection: &mut Connection, tube: String) -> Result<Vec<Data>> {
    connection.watch(tube);
    Ok(vec![
        Data::String("WATCHING".into()),
        Data::Integer(connection.get_watched_tubes().len() as u32),
    ])
}
