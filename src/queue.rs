use std::collections::{HashMap, VecDeque};

use bytes::Bytes;

pub struct Queue {
    num_jobs: u32,
    // TODO: `Connection` struct with watch list
    tubes: HashMap<String, Tube>,
}

#[derive(Default)]
pub struct Tube {
    ready: VecDeque<Job>,

    // TODO: how to handle delays
    delay: VecDeque<Job>,

    /// In original implementation this is a FIFO linked list
    buried: Vec<Job>,
}

#[derive(Debug, PartialEq)]
pub struct Job {
    id: u32,
    ttr: u32,
    pri: u32,
    data: Bytes,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            num_jobs: 0,
            tubes: HashMap::from([("default".to_string(), Tube::default())]),
        }
    }

    pub fn new_job(&mut self, tube: String, ttr: u32, pri: u32, data: Bytes) -> u32 {
        // TODO: Result with following errors:
        //      - "BURIED <id>\r\n" if the server ran out of memory trying to grow the priority
        //      queue data structure.
        //      - <id> is the integer id of the new job
        //      - "EXPECTED_CRLF\r\n" The job body must be followed by a CR-LF pair, that is,
        //      "\r\n". These two bytes are not counted in the job size given by the client in the
        //      put command line.
        //      - "JOB_TOO_BIG\r\n" The client has requested to put a job with a body larger than
        //      max-job-size bytes.
        //      - "DRAINING\r\n" This means that the server has been put into "drain mode" and is
        //      no longer accepting new jobs. The client should try another server or disconnect
        //      and try again later. To put the server in drain mode, send the SIGUSR1 signal to
        //      the process.
        let tube = self.tubes.entry(tube).or_default();
        tube.new_job(self.num_jobs, ttr, pri, data);
        self.num_jobs += 1;
        return self.num_jobs - 1;
    }
}

impl Tube {
    pub fn new_job(&mut self, id: u32, ttr: u32, pri: u32, data: Bytes) {
        if self.ready.is_empty() {
            self.ready.push_back(Job::new(id, ttr, pri, data));
            return;
        }
        let mut index = match self.ready.binary_search_by_key(&pri, |job| job.pri) {
            Ok(i) => i,
            Err(i) => i,
        };
        while index < self.ready.len() && self.ready[index].pri == pri {
            index += 1;
        }
        self.ready.insert(index, Job::new(id, ttr, pri, data));
    }
}

impl Job {
    pub fn new(id: u32, ttr: u32, pri: u32, data: Bytes) -> Self {
        Self { id, ttr, pri, data }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tube_ready() {
        let mut queue = Queue::new();
        // let mut tube = Tube::default();
        queue.new_job("default".to_string(), 0, 0, Bytes::new());
        queue.new_job("default".to_string(), 0, 0, Bytes::new());
        queue.new_job("default".to_string(), 0, 10, Bytes::new());
        queue.new_job("default".to_string(), 0, 1, Bytes::new());
        assert_eq!(
            queue.tubes.get("default").unwrap().ready,
            VecDeque::from([
                Job::new(0, 0, 0, Bytes::new()),
                Job::new(1, 0, 0, Bytes::new()),
                Job::new(3, 0, 1, Bytes::new()),
                Job::new(2, 0, 10, Bytes::new())
            ])
        );
    }
}
