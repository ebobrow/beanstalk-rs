use anyhow::Result;
use bytes::Bytes;

use crate::{codec::Data, connection::Connection};

pub async fn list_tubes_watched(connection: &mut Connection) -> Result<Vec<Data>> {
    let body = format!(
        "---\n{}",
        connection
            .get_watched_tubes()
            .iter()
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
