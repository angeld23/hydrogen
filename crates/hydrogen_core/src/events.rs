use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
struct Event<T> {
    inner: Arc<T>,
    sent_at: Instant,
    index: u32,
}

#[derive(Debug)]
pub struct EventSender<T> {
    pub event_expiration_time: Duration,
    events: Arc<Mutex<VecDeque<Event<T>>>>,
    next_index: Mutex<u32>,
}

impl<T> Default for EventSender<T> {
    fn default() -> Self {
        Self {
            events: Default::default(),
            event_expiration_time: Duration::from_secs(30),
            next_index: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct EventReceiver<T> {
    events: Arc<Mutex<VecDeque<Event<T>>>>,
    next_index: Mutex<u32>,
}

impl<T> EventSender<T> {
    pub fn new(event_expiration_time: Duration) -> Self {
        Self {
            events: Arc::new(Mutex::new(VecDeque::new())),
            event_expiration_time,
            next_index: Mutex::new(0),
        }
    }

    pub fn clean(&self) {
        let mut events = self.events.lock().unwrap();

        loop {
            if let Some(front) = events.front() {
                if front.sent_at.elapsed() > self.event_expiration_time {
                    events.pop_front();
                    continue;
                }
            }

            break;
        }
    }

    pub fn send(&self, event: T) {
        let mut next_index = self.next_index.lock().unwrap();
        let mut events = self.events.lock().unwrap();
        events.push_back(Event {
            inner: Arc::new(event),
            sent_at: Instant::now(),
            index: *next_index,
        });
        *next_index += 1;

        self.clean();
    }

    pub fn subscribe(&self) -> EventReceiver<T> {
        EventReceiver {
            events: Arc::clone(&self.events),
            next_index: Mutex::new(*self.next_index.lock().unwrap()),
        }
    }
}

impl<T> EventReceiver<T> {
    pub fn recv(&self) -> Option<Arc<T>> {
        let mut next_index = self.next_index.lock().unwrap();
        let events = self.events.lock().unwrap();
        let inner = Arc::clone(
            &events
                .iter()
                .find(|event| event.index == *next_index)?
                .inner,
        );

        *next_index += 1;

        Some(inner)
    }

    pub fn recv_all(&self) -> Vec<Arc<T>> {
        let mut result = Vec::<Arc<T>>::new();
        while let Some(event) = self.recv() {
            result.push(event);
        }
        result
    }
}
