use anyhow::bail;
use bytes::Bytes;

use crate::{codec::Data, parser::Parser};

// TODO: macro to parse these :)
enum Cmd {
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

impl TryFrom<Vec<Data>> for Cmd {
    type Error = anyhow::Error;

    fn try_from(data: Vec<Data>) -> Result<Self, Self::Error> {
        let mut parser = Parser::new(data);
        let command_name = parser.consume_name()?;
        match &command_name[..] {
            "put" => Ok(Self::Put {
                pri: parser.consume_integer()?,
                delay: parser.consume_integer()?,
                ttr: parser.consume_integer()?,
                bytes: parser.consume_integer()?,
                data: parser.consume_bytes()?,
            }),
            "use" => Ok(Self::Use {
                tube: parser.consume_name()?,
            }),
            _ => bail!("BAD_FORMAT"),
        }
    }
}
