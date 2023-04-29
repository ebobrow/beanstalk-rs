use bytes::Bytes;

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
