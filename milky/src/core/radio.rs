use crossbeam_queue::SegQueue;

pub struct Radio<T> {
    write_queue: SegQueue<T>,
    read_list: Vec<T>,
}

impl<T> Radio<T> {
    pub fn new() -> Self {
        Self {
            write_queue: SegQueue::new(),
            read_list: vec![],
        }
    }

    pub fn flush(&mut self) {
        let mut read_list = Vec::with_capacity(self.write_queue.len());
        while let Some(val) = self.write_queue.pop() {
            read_list.push(val);
        }
        self.read_list = read_list;
    }

    pub fn send(&self, val: T) {
        self.write_queue.push(val);
    }

    pub fn recv(&self) -> &[T] {
        &self.read_list
    }
}
