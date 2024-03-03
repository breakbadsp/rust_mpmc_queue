use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

pub struct MpmcQueue<T> {
    inner_: Arc<Mutex<VecDeque<T>>>,
    cv_: Condvar,
}

impl<T> MpmcQueue<T> {
    pub fn new() -> Self {
        MpmcQueue {
            inner_: Arc::new(Mutex::new(VecDeque::<T>::new())),
            cv_: Condvar::new(),
        }
    }

    pub fn push(&self, p_data: T) {
        let mut inner = self.inner_.lock().unwrap();
        inner.push_back(p_data);
        self.cv_.notify_one();
    }

    pub fn pop(&self) -> Option<T> {
        let mut inner = self.inner_.lock().unwrap();
        while inner.is_empty() {
            inner = self.cv_.wait(inner).unwrap();
        }
        inner.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_test() {
        let queue = Arc::new(MpmcQueue::new());

        let producer = std::thread::spawn({
            let queue = Arc::clone(&queue);
            move || {
                for i in 0..5 {
                    queue.push(i);
                    println!("Pushed {}", i);
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        });

        let consumer = std::thread::spawn({
            let queue = Arc::clone(&queue);
            move || {
                loop {
                    if let Some(value) = queue.pop() {
                        println!("Popped {}", value);
                    } else {
                        break;
                    }
                }
            }
        });
        
        producer.join().unwrap();
        consumer.join().unwrap();
    }
}