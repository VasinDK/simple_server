use std::sync::{mpsc, Arc, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

/// Структура пула
pub struct ThreadPool {
    /// Элементы вектора состоят из: id потока и дескриптора потока
    workers: Vec<Worker>,
    pub sender: Option<mpsc::Sender<Job>>
}

impl ThreadPool {
    /// Создает пул потоков
    ///
    /// # Arguments
    ///
    /// * `size`: размер пула
    ///
    /// # Panics
    /// * `size` = 0
    ///
    /// returns: ThreadPool
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_server::ThreadPool;
    ///
    /// let pool = ThreadPool::new(10);
    /// ```
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers: workers,
            sender: Some(sender),
        }
    }

    /// Отправляет функцию в поток для выполнения
    ///
    /// # Arguments
    ///
    /// * `f`: функция требующая выполнения
    ///
    /// returns: ()
    ///
    /// # Examples
    ///
    /// ```
    /// use simple_server::ThreadPool;
    ///
    /// let pool = ThreadPool::new(10);
    /// pool.execute(|| {});
    /// ```
    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    /// Изменил метод drop для ThreadPool. Удаляем sender, для выклчения потоков
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

/// Экземпляр потока. Структура в которой есть id и дескриптор потока
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>
}

impl Worker {
    /// Новый экземпляр потока
    ///
    /// Создает новый экземпляр "Работника", в котором указывается id и дескриптор потока
    ///
    /// # Arguments
    ///
    /// * `id`: Идентификатор потока
    /// * `receiver`: Приемник канала
    ///
    /// returns: Worker
    ///
    /// # Examples
    ///
    /// ```Text
    /// use std::sync::{mpsc, Arc, Mutex};
    /// use std::thread;
    ///
    /// let (sender, receiver) = mpsc::channel();
    /// let receiver = Arc::new(Mutex::new(receiver));
    ///
    /// let worker = Worker::new(3, Arc::clone(&receiver));
    /// ```
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => job(),
                Err(_) => break
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn it_pool_panic() {
        // должен паниковать
        ThreadPool::new(0);
    }

    #[test]
    fn it_pool_workers_id() {
        let pool = ThreadPool::new(3);
        assert_eq!(pool.workers[2].id, 2);
    }
}