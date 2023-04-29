use bytes::{BufMut, Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Clone, Debug, PartialEq)]
pub enum Data {
    Name(String),
    Integer(u32),
    Body(Bytes),
}

pub struct Codec {
    next_index: usize,
    frame: Vec<Data>,
}

fn num_from_bytes(buf: &[u8]) -> u32 {
    // TODO: don't like this string allocation
    let num = String::from_utf8(buf.to_vec()).unwrap().parse().unwrap();
    // assert!(num <= 2_u32.pow(32));
    num
}

impl Codec {
    pub fn new() -> Self {
        Self {
            next_index: 0,
            frame: Vec::new(),
        }
    }

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Vec<Data>>, std::io::Error> {
        loop {
            if self.next_index == buf.len() {
                break;
            }
            // TODO: tidy up code
            match buf[self.next_index] {
                c if c.is_ascii_alphabetic() => {
                    if let Some(end) = buf[self.next_index..]
                        .iter()
                        .position(u8::is_ascii_whitespace)
                    {
                        match buf[self.next_index + end] {
                            b' ' => {
                                self.frame.push(Data::Name(
                                    String::from_utf8(
                                        buf[self.next_index..self.next_index + end].to_vec(),
                                    )
                                    .unwrap(),
                                ));
                                self.next_index += end + 1;
                            }
                            b'\r' => {
                                assert_eq!(buf[self.next_index + end + 1], b'\n');
                                self.frame.push(Data::Name(
                                    String::from_utf8(
                                        buf[self.next_index..self.next_index + end].to_vec(),
                                    )
                                    .unwrap(),
                                ));
                                break;
                            }
                            _ => panic!(),
                        }
                    } else {
                        return Ok(None);
                    }
                }
                c if c.is_ascii_digit() => {
                    if let Some(end) = buf[self.next_index..]
                        .iter()
                        .position(u8::is_ascii_whitespace)
                    {
                        match buf[self.next_index + end] {
                            b' ' => {
                                self.frame.push(Data::Integer(num_from_bytes(
                                    &buf[self.next_index..self.next_index + end],
                                )));
                                self.next_index += end + 1;
                            }
                            b'\r' => {
                                assert_eq!(buf[self.next_index + end + 1], b'\n');
                                let num =
                                    num_from_bytes(&buf[self.next_index..self.next_index + end]);
                                self.frame.push(Data::Integer(num));
                                if buf.len() > end + 1 {
                                    self.frame.push(Data::Body(Bytes::copy_from_slice(
                                        &buf[self.next_index + end + 2
                                            ..self.next_index + end + 2 + num as usize],
                                    )));
                                }
                                break;
                            }
                            _ => panic!(),
                        }
                    } else {
                        return Ok(None);
                    }
                }
                _ => unreachable!(),
            }
        }
        // TODO: no clone
        Ok(Some(self.frame.clone()))
    }
}

impl Decoder for Codec {
    type Item = Vec<Data>;

    // TODO: error handling
    type Error = std::io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.decode(buf)
    }
}

impl<T> Encoder<T> for Codec
where
    T: AsRef<str>,
{
    type Error = std::io::Error;

    fn encode(&mut self, item: T, buf: &mut BytesMut) -> Result<(), Self::Error> {
        let item = item.as_ref();
        buf.reserve(item.len() + 1);
        buf.put(item.as_bytes());
        buf.put_u8(b'\n');
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
                Data::Name("put".into()),
                Data::Integer(1),
                Data::Integer(11),
                Data::Integer(101),
                Data::Integer(1),
                Data::Body(Bytes::from_static(b"h"))
            ])
        );
        let mut codec = Codec::new();
        assert_eq!(
            codec
                .decode(&mut BytesMut::from("use default\r\n"))
                .unwrap(),
            Some(vec![Data::Name("use".into()), Data::Name("default".into())])
        );
    }
}
