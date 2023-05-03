use anyhow::{anyhow, bail, Result};
use bytes::{BufMut, Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Clone, Debug, PartialEq)]
pub enum Data {
    String(String),
    Integer(u32),
    Bytes(Bytes),
}

pub struct Codec {
    next_index: usize,
    frame: Vec<Data>,
}

fn string_from_bytes(buf: &[u8]) -> Result<String> {
    String::from_utf8(buf.to_vec()).map_err(|_| anyhow!("INTERNAL_ERROR"))
}

fn num_from_bytes(buf: &[u8]) -> Result<u32> {
    // TODO: don't like this string allocation
    string_from_bytes(buf)?
        .parse()
        .map_err(|_| anyhow!("BAD_FORMAT"))
}

fn valid_name_char(c: u8) -> bool {
    c.is_ascii_digit() || c.is_ascii_alphabetic() || b"-+/;.$_()".contains(&c)
}

impl Codec {
    pub fn new() -> Self {
        Self {
            next_index: 0,
            frame: Vec::new(),
        }
    }

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Vec<Data>>> {
        if buf.is_empty() {
            return Ok(None);
        }
        self.next_index = 0;
        if buf.len() > 8 * 224 {
            bail!("BAD_FORMAT");
        }
        loop {
            let next_char = buf[self.next_index];
            if let Some(end) = buf[self.next_index..]
                .iter()
                .position(u8::is_ascii_whitespace)
            {
                let data = if next_char.is_ascii_digit() {
                    Data::Integer(num_from_bytes(
                        &buf[self.next_index..self.next_index + end],
                    )?)
                } else if valid_name_char(next_char) && next_char != b'-' {
                    if end > 8 * 200
                        || buf[self.next_index..self.next_index + end]
                            .iter()
                            .any(|&c| !valid_name_char(c))
                    {
                        bail!("BAD_FORMAT");
                    }
                    Data::String(string_from_bytes(
                        &buf[self.next_index..self.next_index + end],
                    )?)
                } else {
                    bail!("BAD_FORMAT");
                };
                match buf[self.next_index + end] {
                    b' ' => {
                        self.frame.push(data);
                        self.next_index += end + 1;
                    }
                    b'\r' => {
                        assert_eq!(buf[self.next_index + end + 1], b'\n');
                        let maybe_num = if let Data::Integer(n) = data {
                            Some(n)
                        } else {
                            None
                        };
                        self.frame.push(data);
                        if let Some(num) = maybe_num {
                            if buf.len() > end + 1 {
                                self.frame.push(Data::Bytes(Bytes::copy_from_slice(
                                    &buf[self.next_index + end + 2
                                        ..self.next_index + end + 2 + num as usize],
                                )));
                            }
                        }
                        break;
                    }
                    _ => bail!("BAD_FORMAT"),
                }
            } else {
                return Ok(None);
            }
        }
        // TODO: this seems weird for like batch requests or something
        buf.clear();
        // TODO: no clone
        Ok(Some(self.frame.clone()))
    }
}

impl Decoder for Codec {
    type Item = Vec<Data>;
    type Error = anyhow::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>> {
        self.decode(buf)
    }
}

impl Encoder<Vec<Data>> for Codec {
    type Error = anyhow::Error;

    fn encode(
        &mut self,
        data: Vec<Data>,
        buf: &mut BytesMut,
    ) -> std::result::Result<(), Self::Error> {
        // TODO: hideous fence post??
        let mut i = 0;
        let length = data.len();
        for item in data {
            i += 1;
            match item {
                Data::String(name) => buf.put(name.as_bytes()),
                Data::Integer(n) => buf.put(n.to_string().as_bytes()),
                Data::Bytes(bytes) => buf.put(bytes),
            }
            if i < length {
                buf.put_u8(b' ');
            }
        }
        buf.put_slice(b"\r\n");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes() {
        let mut codec = Codec::new();
        assert_eq!(
            codec
                .decode(&mut BytesMut::from("put 1 11 101 1\r\nh\r\n"))
                .unwrap(),
            Some(vec![
                Data::String("put".into()),
                Data::Integer(1),
                Data::Integer(11),
                Data::Integer(101),
                Data::Integer(1),
                Data::Bytes(Bytes::from_static(b"h"))
            ])
        );
        let mut codec = Codec::new();
        assert_eq!(
            codec
                .decode(&mut BytesMut::from("use default+$23\r\n"))
                .unwrap(),
            Some(vec![
                Data::String("use".into()),
                Data::String("default+$23".into())
            ])
        );
    }

    #[test]
    fn int_too_big() {
        let mut codec = Codec::new();
        if let Err(e) = codec.decode(&mut BytesMut::from("4294967296\r\n")) {
            assert_eq!(e.to_string(), "BAD_FORMAT");
        } else {
            panic!("did not error");
        }
    }

    #[test]
    fn invalid_name() {
        let mut codec = Codec::new();
        if let Err(e) = codec.decode(&mut BytesMut::from("-name\r\n")) {
            assert_eq!(e.to_string(), "BAD_FORMAT");
        } else {
            panic!("did not error");
        }

        let mut codec = Codec::new();
        if let Err(e) = codec.decode(&mut BytesMut::from("name^\r\n")) {
            assert_eq!(e.to_string(), "BAD_FORMAT");
        } else {
            panic!("did not error");
        }
    }

    #[test]
    fn too_long() {
        let mut codec = Codec::new();
        if let Err(e) = codec.decode(&mut BytesMut::from(
            &format!("{}\r\n", "a".repeat(8 * 224))[..],
        )) {
            assert_eq!(e.to_string(), "BAD_FORMAT");
        } else {
            panic!("did not error");
        }

        let mut codec = Codec::new();
        if let Err(e) = codec.decode(&mut BytesMut::from(
            &format!("put {}\r\n", "a".repeat(8 * 201))[..],
        )) {
            assert_eq!(e.to_string(), "BAD_FORMAT");
        } else {
            panic!("did not error");
        }
    }
}
