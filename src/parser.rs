use anyhow::{bail, Result};
use bytes::Bytes;

use crate::codec::Data;

pub struct Parser {
    data: std::vec::IntoIter<Data>,
}

impl Parser {
    pub fn new(data: Vec<Data>) -> Self {
        Self {
            data: data.into_iter(),
        }
    }

    pub fn consume_name(&mut self) -> Result<String> {
        if let Some(Data::String(name)) = self.data.next() {
            Ok(name)
        } else {
            bail!("BAD_FORMAT");
        }
    }

    pub fn consume_integer(&mut self) -> Result<u32> {
        if let Some(Data::Integer(i)) = self.data.next() {
            Ok(i)
        } else {
            bail!("BAD_FORMAT");
        }
    }

    pub fn consume_bytes(&mut self) -> Result<Bytes> {
        if let Some(Data::Bytes(b)) = self.data.next() {
            Ok(b)
        } else {
            bail!("BAD_FORMAT");
        }
    }

    pub fn finish(&mut self) -> Result<()> {
        if self.data.next().is_none() {
            Ok(())
        } else {
            bail!("BAD_FORMAT");
        }
    }
}
