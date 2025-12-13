use std::{
    collections::{BTreeMap, VecDeque},
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
    named_receivers: Arc<Mutex<BTreeMap<String, EventReceiver<T>>>>,
    next_index: Mutex<u32>,
}

impl<T> Default for EventSender<T> {
    fn default() -> Self {
        Self {
            event_expiration_time: Duration::from_secs(30),
            events: Default::default(),
            named_receivers: Default::default(),
            next_index: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct EventReceiver<T> {
    events: Arc<Mutex<VecDeque<Event<T>>>>,
    next_index: Mutex<u32>,
}

impl<T> Clone for EventReceiver<T> {
    fn clone(&self) -> Self {
        Self {
            events: self.events.clone(),
            next_index: (*self.next_index.try_lock().unwrap()).into(),
        }
    }
}

impl<T> EventSender<T> {
    pub fn new(event_expiration_time: Duration) -> Self {
        Self {
            event_expiration_time,
            ..Default::default()
        }
    }

    pub fn clean(&self) {
        {
            let mut events = self.events.try_lock().unwrap();
            loop {
                if let Some(front) = events.front()
                    && front.sent_at.elapsed() > self.event_expiration_time
                {
                    events.pop_front();
                    continue;
                }

                break;
            }
        }

        self.named_receivers
            .try_lock()
            .unwrap()
            .retain(|_, receiver| receiver.peek().is_some());
    }

    pub fn send(&self, event: impl Into<Arc<T>>) {
        {
            let mut next_index = self.next_index.try_lock().unwrap();
            let mut events = self.events.try_lock().unwrap();
            events.push_back(Event {
                inner: event.into(),
                sent_at: Instant::now(),
                index: *next_index,
            });
            *next_index += 1;
        }

        self.clean();
    }

    pub fn subscribe(&self) -> EventReceiver<T> {
        EventReceiver {
            events: Arc::clone(&self.events),
            next_index: Mutex::new(*self.next_index.try_lock().unwrap()),
        }
    }

    pub fn named_receiver(&self, name: impl Into<String>) -> EventReceiver<T> {
        let name = name.into();
        let mut named_receivers = self.named_receivers.try_lock().unwrap();
        if let Some(receiver) = named_receivers.get(&name) {
            return receiver.clone();
        }

        let receiver = self.subscribe();
        named_receivers.insert(name, receiver.clone());
        receiver
    }

    pub fn receiver_count(&self) -> u32 {
        (Arc::strong_count(&self.events) as u32).saturating_sub(1)
    }
}

impl<T> EventReceiver<T> {
    pub fn peek(&self) -> Option<Arc<T>> {
        let next_index = self.next_index.try_lock().unwrap();
        let events = self.events.try_lock().unwrap();
        let inner = Arc::clone(
            &events
                .iter()
                .find(|event| event.index == *next_index)?
                .inner,
        );

        Some(inner)
    }

    pub fn recv(&self) -> Option<Arc<T>> {
        let inner = self.peek()?;

        let mut next_index = self.next_index.try_lock().unwrap();
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
