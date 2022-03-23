use std::sync::{
    atomic::{AtomicBool, Ordering as AtomicOrd},
    mpsc::{self, SendError},
    Arc,
};
use std::thread::{spawn, JoinHandle};

pub struct ThreadPool {
    capacity: Option<usize>,
    workers: Vec<Worker>,
}

impl ThreadPool {
    pub fn new() -> Self {
        ThreadPool {
            capacity: None,
            workers: Vec::new(),
        }
    }

    pub fn submit(&mut self, job: impl FnOnce() + Sync + Send + 'static) -> Result<(), ()> {
        let free_worker = self
            .workers
            .iter_mut()
            .find(|w| !w.busy.load(AtomicOrd::Relaxed));

        match free_worker {
            Some(w) => {
                w.send(Message::NewJob(Box::new(job))).map_err(|_| ())?;
            }

            None => match self.capacity {
                Some(c) if self.workers.len() < c => {
                    let mut w = Worker::new();
                    w.send(Message::NewJob(Box::new(job))).map_err(|_| ())?;
                    self.workers.push(w);
                }

                Some(_) => {
                    use rand::random;
                    let choice = random::<usize>() % self.workers.len();
                    self.workers[choice]
                        .send(Message::NewJob(Box::new(job)))
                        .map_err(|_| ())?;
                }

                None => {
                    let mut w = Worker::new();
                    w.send(Message::NewJob(Box::new(job))).map_err(|_| ())?;
                    self.workers.push(w);
                }
            },
        }

        Ok(())
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Shutting Down Thread Pool!");
        for worker in &mut self.workers {
            worker.send(Message::Terminate).unwrap();
            worker.join().unwrap();
        }
    }
}

struct Worker {
    busy: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    sender: mpsc::Sender<Message>,
}

impl Worker {
    fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let busy = Arc::new(AtomicBool::new(false));

        let mut worker = Worker {
            busy: busy.clone(),
            handle: None,
            sender,
        };

        worker.handle = Some(spawn(|| Worker::handler(receiver, busy)));

        return worker;
    }

    fn send(&mut self, msg: Message) -> Result<(), SendError<Message>> {
        self.sender.send(msg)?;
        return Ok(());
    }

    fn join(&mut self) -> Result<(), ()> {
        match self.handle.take() {
            Some(j) => {
                j.join().map_err(|_| ())?;
                Ok(())
            }

            None => Ok(()),
        }
    }

    fn handler(receiver: mpsc::Receiver<Message>, busy: Arc<AtomicBool>) {
        loop {
            busy.store(false, AtomicOrd::Relaxed);
            match receiver.recv().unwrap() {
                Message::Terminate => return,
                Message::NewJob(j) => {
                    busy.store(true, AtomicOrd::Relaxed);
                    j();
                }
            }
        }
    }
}

enum Message {
    Terminate,
    NewJob(Box<dyn FnOnce() + Send + Sync>),
}
