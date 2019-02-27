use std::fmt;

use slab::Slab;


pub struct Subscription {
    key: usize,
}


#[derive(Debug)]
pub struct SubscriptionMissing;

impl fmt::Display for SubscriptionMissing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Attempt to unsubscribe delegate without subscription")
    }
}

impl std::error::Error for SubscriptionMissing {}


pub struct Event<T> {
    handlers: Slab<Box<Fn(T) + Send + Sync>>,
}

impl<T> Default for Event<T> {
    fn default() -> Self {
        Event {
            handlers: Slab::new(),
        }
    }
}

impl<T> Event<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        Event::default()
    }

    pub fn subscribe<F>(&mut self, handler: F) -> Subscription
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        Subscription {
            key: self.handlers.insert(Box::new(handler)),
        }
    }

    pub fn unsubscribe(
        &mut self,
        subscription: Subscription,
    ) -> Result<(), SubscriptionMissing> {
        if self.handlers.contains(subscription.key) {
            self.handlers.remove(subscription.key);
            Ok(())
        } else {
            Err(SubscriptionMissing)
        }
    }

    pub fn emit(&self, args: T) {
        for (_, handler) in &self.handlers {
            (*handler)(args.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::{Arc, Mutex};

    #[test]
    fn test_event() {
        let called1 = Arc::new(Mutex::new(false));
        let called2 = Arc::new(Mutex::new(false));
        let some_buffer = Arc::new(Mutex::new(Vec::new()));

        {
            let mut my_event = Event::<u8>::new();
            let called1 = called1.clone();
            my_event.subscribe(move |x| {
                *called1.lock().unwrap() = true;
                assert_eq!(x, 42);
            });
            let called2 = called2.clone();
            let some_buffer = some_buffer.clone();
            my_event.subscribe(move |x| {
                *called2.lock().unwrap() = true;
                some_buffer.lock().unwrap().push(x);
            });

            my_event.emit(42);
        }

        assert!(*called1.lock().unwrap());
        assert!(*called2.lock().unwrap());
        assert_eq!(*some_buffer.lock().unwrap(), vec![42]);
    }

    #[test]
    fn test_unsubscribe() {
        let called = Arc::new(Mutex::new(0u8));
        {
            let mut my_event = Event::<()>::new();
            let called = called.clone();
            let subscription = my_event.subscribe(move |_| {
                *called.lock().unwrap() += 1;
            });
            my_event.emit(());
            my_event.emit(());
            my_event.unsubscribe(subscription).unwrap();
            my_event.emit(());
            my_event.emit(());
        }
        assert_eq!(*called.lock().unwrap(), 2);
    }
}
