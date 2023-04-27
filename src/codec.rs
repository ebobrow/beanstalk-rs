// TODO
// use bytes::{Bytes, BytesMut};
// use tokio_util::codec::Decoder;

// enum Data {
//     Name(String),
//     Integer(u32),
//     Body(Bytes),
// }

// struct Frame {
//     data: Vec<Data>,
// }

// struct Codec;

// impl Decoder for Codec {
//     type Item = Frame;

//     // TODO: error handling
//     type Error = std::io::Error;

//     fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
//         todo!()
//     }
// }
