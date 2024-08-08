use std::{
  sync::{mpsc, Arc, Mutex},
  thread,
};

pub struct ThreadPool {
  workers: Vec<Worker>,
  sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
  /// Create a new ThreadPool.
  ///
  /// The size is the number of threads in the pool.
  ///
  /// # Panics
  ///
  /// The `new` function will panic if the size is zero.
  pub fn new(size: usize) -> Self {
    assert!(size > 0);

    let (sender, receiver) = mpsc::channel();

    let receiver = Arc::new(Mutex::new(receiver));

    let mut workers = Vec::with_capacity(size);

    for _ in 0..size {
      workers.push(Worker::new(Arc::clone(&receiver)));
    }

    Self { workers, sender: Some(sender) }
  }

  pub fn execute<F>(&self, f: F)
  where
    F: FnOnce() + Send + 'static,
  {
    let job = Box::new(f);
    self.sender.as_ref().unwrap().send(job).unwrap();
  }
}

impl Default for ThreadPool {
  fn default() -> Self {
    Self::new(num_cpus::get())
  }
}

impl Drop for ThreadPool {
  fn drop(&mut self) {
    drop(self.sender.take());

    for worker in &mut self.workers {
      // #[cfg(debug_assertions)]
      // println!("Shutting down worker {}", worker.id);

      if let Some(thread) = worker.thread.take() {
        thread.join().unwrap();
      }
    }
  }
}

struct Worker {
  thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
  fn new(receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
    let thread = thread::spawn(move || loop {
      let job = receiver.lock().unwrap().recv();

      match job {
        Ok(job) => job(),
        Err(_) => break,
      }
    });

    Self { thread: Some(thread) }
  }
}
