use futures_util::StreamExt;
use std::collections::{HashMap, VecDeque};

use bytes::Bytes;
use futures_util::stream::FuturesUnordered;
use tokio::{
    select,
    sync::mpsc,
    time::{sleep, Duration},
};

pub struct Queue {
    // TODO: `Connection` struct with watch list
    tubes: HashMap<String, Tube>,
    jobs: Vec<Job>,
    new_job_tx: mpsc::Sender<(String, u32, u32)>,
}

#[derive(Default)]
pub struct Tube {
    ready: VecDeque<u32>,
    delay: Vec<u32>,
    smallest_pri: u32,

    /// In original implementation this is a FIFO linked list
    buried: Vec<u32>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Job {
    pub id: u32,
    pub ttr: u32,
    pub pri: u32,
    pub data: Bytes,
}

impl Queue {
    pub fn new(ready_job_tx: mpsc::Sender<(String, u32)>) -> Self {
        let (new_job_tx, new_job_rx) = mpsc::channel(100);
        // This is an implementation detail that differs from the original Beanstalk. Instead of each
        // tube having a delay queue, they are all in this one to make async polling easier.
        tokio::spawn(watch_delayed_jobs(new_job_rx, ready_job_tx));
        Self {
            tubes: HashMap::from([("default".to_string(), Tube::default())]),
            jobs: Vec::new(),
            new_job_tx,
        }
    }

    fn job(&self, id: &u32) -> &Job {
        self.jobs.iter().find(|job| &job.id == id).unwrap()
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
        let id = self.jobs.len() as u32 + 1;
        self.jobs.push(Job::new(id, ttr, pri, data));
        self.queue_job(tube, id);
        id
    }

    pub async fn new_delayed_job(
        &mut self,
        tube: String,
        ttr: u32,
        pri: u32,
        delay: u32,
        data: Bytes,
    ) -> u32 {
        let id = self.jobs.len() as u32 + 1;
        self.jobs.push(Job::new(id, ttr, pri, data));
        self.new_tube(&tube).delay.push(id);
        self.new_job_tx.send((tube, id, delay)).await.unwrap();
        id
    }

    pub fn queue_job(&mut self, tube: String, id: u32) {
        // TODO: why error when use `self.job(&id)`?
        // also just store `&Job`?
        let job = self.jobs.iter().find(|job| job.id == id).unwrap();
        let mut tube = self.tubes.entry(tube).or_default();
        if let Some((idx, _)) = tube.delay.iter().enumerate().find(|(_, job)| job == &&id) {
            tube.delay.remove(idx);
        }
        if tube.ready.is_empty() {
            tube.ready.push_back(id);
            tube.smallest_pri = job.pri;
            return;
        }
        let mut index = match tube.ready.binary_search_by_key(&job.pri, |id| {
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
                == job.pri
        {
            index += 1;
        }
        tube.ready.insert(index, id);
        if index == 0 {
            tube.smallest_pri = job.pri;
        }
    }

    pub fn delete_job(&mut self, id: u32) -> bool {
        if let Some((i, _)) = self.jobs.iter().enumerate().find(|(_, job)| job.id == id) {
            self.jobs.remove(i);
            true
        } else {
            false
        }
    }

    pub fn reserve_job(&mut self, watch_list: Vec<String>) -> Option<&Job> {
        let name = watch_list
            .iter()
            .min_by_key(|&name| self.tubes.get(name).unwrap().smallest_pri)
            .unwrap();
        let tube = self.tubes.get_mut(name).unwrap();
        tube.ready.pop_front().map(|id| self.job(&id))
    }

    pub fn reserve_by_id(&mut self, id: u32) -> Option<&Job> {
        let job = self.jobs.iter().find(|job| job.id == id);
        for (_, tube) in &mut self.tubes {
            if let Some((idx, _)) = tube.ready.iter().enumerate().find(|(_, job)| job == &&id) {
                tube.ready.remove(idx);
                return job;
            } else if let Some((idx, _)) =
                tube.buried.iter().enumerate().find(|(_, job)| job == &&id)
            {
                tube.buried.remove(idx);
                return job;
            } else if let Some((idx, _)) =
                tube.delay.iter().enumerate().find(|(_, job)| job == &&id)
            {
                tube.delay.remove(idx);
                return job;
            }
        }
        None
    }

    pub fn tube_names(&self) -> std::collections::hash_map::Keys<String, Tube> {
        self.tubes.keys()
    }
}

impl Job {
    pub fn new(id: u32, ttr: u32, pri: u32, data: Bytes) -> Self {
        let ttr = if ttr == 0 { 1 } else { ttr };
        Self { id, ttr, pri, data }
    }
}

async fn watch_delayed_jobs(
    mut new_job_rx: mpsc::Receiver<(String, u32, u32)>,
    ready_job_tx: mpsc::Sender<(String, u32)>,
) {
    let mut jobs = FuturesUnordered::new();
    loop {
        select! {
            Some((tube, id, delay)) = new_job_rx.recv() => {
                jobs.push(tokio::spawn(async move {
                    sleep(Duration::from_secs(delay as u64)).await;
                    (tube, id)
                }));
            }
            Some(Ok(job)) = jobs.next() => {
                ready_job_tx.send(job).await.unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn tube_ready() {
        let (ready_job_tx, _ready_job_rx) = mpsc::channel(100);
        let mut queue = Queue::new(ready_job_tx);
        queue.new_job("default".to_string(), 0, 0, Bytes::new());
        queue.new_job("default".to_string(), 0, 0, Bytes::new());
        queue.new_job("default".to_string(), 0, 10, Bytes::new());
        queue.new_job("default".to_string(), 0, 1, Bytes::new());
        assert_eq!(
            queue.tubes.get("default").unwrap().ready,
            VecDeque::from([1, 2, 4, 3])
        );
    }

    #[tokio::test]
    async fn smallest_pri() {
        let (ready_job_tx, _ready_job_rx) = mpsc::channel(100);
        let mut queue = Queue::new(ready_job_tx);
        queue.new_job("default".to_string(), 0, 100, Bytes::new());
        assert_eq!(queue.tubes.get("default").unwrap().smallest_pri, 100);
        queue.new_job("default".to_string(), 0, 1000, Bytes::new());
        assert_eq!(queue.tubes.get("default").unwrap().smallest_pri, 100);
        queue.new_job("default".to_string(), 0, 10, Bytes::new());
        assert_eq!(queue.tubes.get("default").unwrap().smallest_pri, 10);
        queue.new_job("default".to_string(), 0, 1, Bytes::new());
        assert_eq!(queue.tubes.get("default").unwrap().smallest_pri, 1);
    }

    #[tokio::test]
    #[ignore]
    async fn delay_job() {
        use std::sync::{Arc, Mutex};

        let (ready_job_tx, mut ready_job_rx) = mpsc::channel(100);
        let done = Arc::new(Mutex::new(false));

        let done1 = done.clone();
        tokio::spawn(async move {
            if let Some(_) = ready_job_rx.recv().await {
                let mut done = done1.lock().unwrap();
                *done = true;
            }
        });

        let mut queue = Queue::new(ready_job_tx);
        queue
            .new_delayed_job("default".to_string(), 0, 0, 1, Bytes::new())
            .await;
        {
            let done = done.lock().unwrap();
            assert!(!*done);
        }
        sleep(Duration::from_secs_f32(1.1)).await;
        {
            let done = done.lock().unwrap();
            assert!(*done);
        }
    }
}
