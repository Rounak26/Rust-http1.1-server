use std::net::TcpListener;
use std::sync::mpsc::{self, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::config::Config;
use crate::error::Result;
use crate::router::Router;
use crate::server::connection;

pub struct Server {
    config: Arc<Config>,
    router: Arc<Router>,
}

impl Server {
    pub fn new(config: Config, router: Router) -> Self {
        Server {
            config: Arc::new(config),
            router: Arc::new(router),
        }
    }
    
    pub fn run(self) -> Result<()> {
        let listener = TcpListener::bind((self.config.host.as_str(), self.config.port))?;
        println!("listening on http://{}", listener.local_addr()?);
        self.serve(listener)
    }

    pub fn serve(self, listener: TcpListener) -> Result<()> {
        let pool = ThreadPool::new(self.config.workers);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let router = Arc::clone(&self.router);
                    let config = Arc::clone(&self.config);
                    pool.execute(move || connection::handle(stream, router, config));
                }
                Err(err) => eprintln!("accept failed: {err}"),
            }
        }
        Ok(())
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPool {
    workers: Vec<JoinHandle<()>>,
    sender: Option<SyncSender<Job>>,
}

impl ThreadPool {
    fn new(size: usize) -> Self {
        let size = size.max(1);
        let (sender, receiver) = mpsc::sync_channel::<Job>(size * 4);
        let receiver = Arc::new(Mutex::new(receiver));

        let workers = (0..size)
            .map(|id| {
                let receiver = Arc::clone(&receiver);
                thread::Builder::new()
                    .name(format!("worker-{id}"))
                    .spawn(move || loop {
                        let job = receiver.lock().unwrap().recv();
                        match job {
                            Ok(job) => job(),
                            Err(_) => break, // sender dropped: shut down
                        }
                    })
                    .expect("failed to spawn worker thread")
            })
            .collect();

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    fn execute<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        if let Some(sender) = &self.sender {
            let _ = sender.send(Box::new(job));
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}
