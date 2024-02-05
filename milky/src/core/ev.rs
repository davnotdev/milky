use std::sync::{
    mpsc::{channel, Receiver, Sender, TryIter as ReceiverIter, TryRecvError},
    Mutex,
};

pub struct Ev<T> {
    tx: Sender<T>,
    rx: Mutex<Option<Receiver<T>>>,
}

impl<T> Ev<T> {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            tx,
            rx: Mutex::new(Some(rx)),
        }
    }

    pub fn take(&self) -> Recv<T> {
        self.rx
            .lock()
            .unwrap()
            .take()
            .map(|recv| Recv { rx: recv })
            .unwrap_or_else(|| panic!("Cannot take twice"))
    }

    pub fn send(&self, data: T) {
        self.tx.send(data).unwrap();
    }
}

pub struct Recv<T> {
    rx: Receiver<T>,
}

impl<T> Recv<T> {
    pub fn try_recv(&self) -> Option<T> {
        match self.rx.try_recv() {
            Ok(data) => Some(data),
            Err(TryRecvError::Empty) => None,
            _ => panic!("Disconnected sender somewhere"),
        }
    }

    pub fn iter(&self) -> ReceiverIter<T> {
        self.rx.try_iter()
    }
}
