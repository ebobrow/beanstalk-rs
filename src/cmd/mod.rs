use std::sync::Arc;

use anyhow::Result;
use bytes::Bytes;
use macros::Parse;
use tokio::sync::Mutex;

use crate::{codec::Data, connection::Connection, queue::Queue};

mod delete;
mod ignore;
mod list_tube_used;
mod list_tubes;
mod list_tubes_watched;
mod put;
mod quit;
mod reserve;
mod r#use;
mod watch;

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

// TODO: test each of these (https://rust-lang.github.io/async-book/09_example/03_tests.html)
impl Cmd {
    pub async fn run(
        self,
        connection: &mut Connection,
        queue: Arc<Mutex<Queue>>,
    ) -> Result<Vec<Data>> {
        match self {
            Cmd::Put {
                pri,
                delay,
                ttr,
                bytes: _,
                data,
            } => put::put(connection, queue, pri, delay, ttr, data).await,
            Cmd::Use { tube } => r#use::use_tube(connection, queue, tube).await,
            Cmd::Reserve => reserve::reserve_with_timeout(connection, queue, 0).await,
            Cmd::ReserveWithTimeout { seconds } => {
                reserve::reserve_with_timeout(connection, queue, seconds).await
            }
            Cmd::ReserveJob { id } => todo!(),
            Cmd::Delete { id } => delete::delete(connection, queue, id).await,
            Cmd::Release { id, pri, delay } => todo!(),
            Cmd::Bury { id, pri } => todo!(),
            Cmd::Touch { id } => todo!(),
            Cmd::Watch { tube } => watch::watch(connection, tube),
            Cmd::Ignore { tube } => ignore::ignore(connection, tube),
            Cmd::Peek { id } => todo!(),
            Cmd::PeekReady => todo!(),
            Cmd::PeekDelayed => todo!(),
            Cmd::PeekBuried => todo!(),
            Cmd::Kick { bound } => todo!(),
            Cmd::KickJob { id } => todo!(),
            Cmd::StatsJob { id } => todo!(),
            Cmd::StatsTube { tube } => todo!(),
            Cmd::Stats => todo!(),
            Cmd::ListTubes => list_tubes::list_tubes(queue).await,
            Cmd::ListTubeUsed => list_tube_used::list_tube_used(connection).await,
            Cmd::ListTubesWatched => list_tubes_watched::list_tubes_watched(connection).await,
            Cmd::Quit => quit::quit(connection),
            Cmd::PauseTube { tube_name, delay } => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::codec::Data;

    use super::*;

    #[test]
    fn parse() {
        let data = vec![
            Data::String("put".into()),
            Data::Integer(1),
            Data::Integer(2),
            Data::Integer(3),
            Data::Integer(4),
            Data::Bytes(Bytes::from_static(b"hello")),
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
