use bytes::Bytes;
use macros::Parse;

// TODO: macro to parse these :)
#[derive(Parse, PartialEq, Debug)]
pub enum Cmd {
    Put {
        pri: u32,
        delay: u32,
        ttr: u32,
        bytes: u32,
        data: Bytes, // or [u8]
    },
    Use {
        tube: String,
    },
    Reserve,
    ReserveWithTimeout {
        seconds: u32,
    },
    ReserveJob {
        id: u32,
    },
    Delete {
        id: u32,
    },
    Release {
        id: u32,
        pri: u32,
        delay: u32,
    },
    Bury {
        id: u32,
        pri: u32,
    },
    Touch {
        id: u32,
    },
    Watch {
        tube: String,
    },
    Ignore {
        tube: String,
    },
    Peek {
        id: u32,
    },
    PeekReady,
    PeekDelayed,
    PeekBuried,
    Kick {
        bound: u32,
    },
    KickJob {
        id: u32,
    },
    StatsJob {
        id: u32,
    },
    StatsTube {
        tube: String,
    },
    Stats,
    ListTubes,
    ListTubeUsed,
    ListTubesWatched,
    Quit,
    PauseTube {
        tube_name: String,
        delay: u32,
    },
}

#[cfg(test)]
mod tests {
    use crate::codec::Data;

    use super::*;

    #[test]
    fn parse() {
        let data = vec![
            Data::Name("put".into()),
            Data::Integer(1),
            Data::Integer(2),
            Data::Integer(3),
            Data::Integer(4),
            Data::Body(Bytes::from_static(b"hello")),
        ];
        assert_eq!(
            Cmd::try_from(data).unwrap(),
            Cmd::Put {
                pri: 1,
                delay: 2,
                ttr: 3,
                bytes: 4,
                data: Bytes::from_static(b"hello")
            }
        );
    }
}
