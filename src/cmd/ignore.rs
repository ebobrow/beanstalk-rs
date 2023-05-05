use anyhow::Result;

use crate::{codec::Data, connection::Connection};

pub fn ignore(connection: &mut Connection, tube: String) -> Result<Vec<Data>> {
    let watched_tubes = connection.get_watched_tubes();
    if watched_tubes.len() == 1 && watched_tubes.contains(&tube) {
        Ok(vec![Data::String("NOT_IGNORED".into())])
    } else {
        connection.ignore(tube);
        Ok(vec![
            Data::String("WATCHING".into()),
            Data::Integer(connection.get_watched_tubes().len() as u32),
        ])
    }
}
