use std::thread::{spawn, JoinHandle};
use std::sync::{mpsc};

pub struct ThreadPool {
    capacity: usize,
    workers: Vec<Worker>,
    next_worker: usize,
}

impl ThreadPool {
    pub fn new(capacity: usize) -> Self {
        let mut workers = Vec::with_capacity(capacity);
        for _i in 0..capacity {
            workers.push(Worker::new());
        }

        ThreadPool {
            capacity,
            workers,
            next_worker: 0,
        }
    }

    pub fn submit(&mut self, job: impl FnOnce() + Sync + Send + 'static) -> Result<(), ()> {
        self.workers[self.next_worker].sender.send(Message::NewJob(Box::new(job))).map_err(|_| ())?;
        self.next_worker = (self.next_worker + 1) % self.capacity;
        return Ok(())
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
    handle: Option<JoinHandle<()>>,
    sender: mpsc::Sender<Message>,
}

impl Worker {
    fn new() -> Self {
        let (sender, receiver) = mpsc::channel();

        let mut worker = Worker {
            handle: None,
            sender,
        };

        worker.handle = Some(spawn(|| Worker::handler(receiver)));

        return worker;
    }

    fn send(&mut self, msg: Message) -> Result<(), ()> {
        self.sender.send(msg).map_err(|_| ())?;
        return Ok(());
    }

    fn join(&mut self) -> Result<(), ()> {
        match self.handle.take() {
            Some(j) => {
                j.join().map_err(|_| ())?;
                Ok(())
            },

            None => Ok(()),
        }
    }

    fn handler(receiver: mpsc::Receiver<Message>) {
        loop {
            match receiver.recv().unwrap() {
                Message::Terminate => return,
                Message::NewJob(j) => j(),
            }
        }
    }
}

enum Message {
    Terminate,
    NewJob(Box<dyn FnOnce() + Send + Sync>)
}