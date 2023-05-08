use anyhow::Result;

use crate::{codec::Data, connection::Connection};

pub fn quit(connection: &mut Connection) -> Result<Vec<Data>> {
    connection.quit();
    Ok(Vec::new())
}
