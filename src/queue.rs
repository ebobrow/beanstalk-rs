use std::collections::{HashMap, VecDeque};

use bytes::Bytes;

pub struct Queue {
    // TODO: `Connection` struct with watch list
    tubes: HashMap<String, Tube>,
    jobs: Vec<Job>,
}

#[derive(Default)]
pub struct Tube {
    ready: VecDeque<u32>,

    // TODO: how to handle delays
    delay: VecDeque<u32>,

    /// In original implementation this is a FIFO linked list
    buried: Vec<u32>,
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
            tubes: HashMap::from([("default".to_string(), Tube::default())]),
            jobs: Vec::new(),
        }
    }

    pub fn new_tube(&mut self, tube: impl ToString) -> &mut Tube {
        self.tubes.entry(tube.to_string()).or_default()
    }

    pub fn new_job(&mut self, tube: String, ttr: u32, pri: u32, data: Bytes) -> u32 {
        // TODO: Result with following errors:
        //      - "BURIED <id>\r\n" if the server ran out of memory trying to grow the priority
        //      queue data structure.
        //          - <id> is the integer id of the new job
        //      - "DRAINING\r\n" This means that the server has been put into "drain mode" and is
        //      no longer accepting new jobs. The client should try another server or disconnect
        //      and try again later. To put the server in drain mode, send the SIGUSR1 signal to
        //      the process.
        let id = self.jobs.len() as u32;
        self.jobs.push(Job::new(id, ttr, pri, data));
        let tube = self.tubes.entry(tube.to_string()).or_default();
        if tube.ready.is_empty() {
            tube.ready.push_back(id);
            return id;
        }
        let mut index = match tube.ready.binary_search_by_key(&pri, |id| {
            self.jobs.iter().find(|job| &job.id == id).unwrap().pri
        }) {
            Ok(i) => i,
            Err(i) => i,
        };
        while index < tube.ready.len()
            && self
                .jobs
                .iter()
                .find(|job| job.id == tube.ready[index])
                .unwrap()
                .pri
                == pri
        {
            index += 1;
        }
        tube.ready.insert(index, id);
        id
    }

    pub fn delete_job(&mut self, id: u32) -> bool {
        if let Some((i, _)) = self.jobs.iter().enumerate().find(|(_, job)| job.id == id) {
            self.jobs.remove(i);
            true
        } else {
            false
        }
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
        queue.new_job("default".to_string(), 0, 0, Bytes::new());
        queue.new_job("default".to_string(), 0, 0, Bytes::new());
        queue.new_job("default".to_string(), 0, 10, Bytes::new());
        queue.new_job("default".to_string(), 0, 1, Bytes::new());
        assert_eq!(
            queue.tubes.get("default").unwrap().ready,
            VecDeque::from([0, 1, 3, 2])
        );
    }
}
