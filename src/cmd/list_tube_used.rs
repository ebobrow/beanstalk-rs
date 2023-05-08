use anyhow::Result;

use crate::{codec::Data, connection::Connection};

pub async fn list_tube_used(connection: &mut Connection) -> Result<Vec<Data>> {
    Ok(vec![
        Data::String("USING".into()),
        Data::String(connection.tube().to_string()),
    ])
}
