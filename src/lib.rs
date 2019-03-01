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


macro_rules! event_impl {
    ($name:ident => $fn:tt, [$($bound:tt),*], $self:ident: $self_ty:ty, $iter_ex:expr) => {
        pub struct $name<'a, T> {
            handlers: Slab<Box<$fn(T) $( + $bound)* + 'a>>,
        }

        impl<'a, T> Default for $name<'a, T> {
            fn default() -> Self {
                $name {
                    handlers: Slab::new(),
                }
            }
        }

        impl<'a, T> $name<'a, T>
        where
            T: Clone,
        {
            pub fn new() -> Self {
                $name::default()
            }

            pub fn subscribe<F>(&mut self, handler: F) -> Subscription
            where
                F: $fn(T) $( + $bound)* + 'a,
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

            pub fn emit($self:$self_ty, args: T) {
                for (_, handler) in $iter_ex {
                    (*handler)(args.clone())
                }
            }
        }
    };
}

event_impl!(Event => Fn, [], self: &Self, &self.handlers);
event_impl!(SyncEvent => Fn, [Send, Sync], self: &Self, &self.handlers);
event_impl!(MutEvent => FnMut, [], self: &mut Self, &mut self.handlers);

pub type StaticEvent<T> = Event<'static, T>;
pub type StaticSyncEvent<T> = SyncEvent<'static, T>;
pub type StaticMutEvent<T> = MutEvent<'static, T>;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event() {
        let mut called1 = false;
        let mut called2 = false;
        let mut some_buffer = Vec::new();
        {
            let mut my_event = MutEvent::<u8>::new();
            my_event.subscribe(|x| {
                called1 = true;
                assert_eq!(x, 42);
            });
            my_event.subscribe(|x| {
                called2 = true;
                some_buffer.push(x);
            });

            my_event.emit(42);
        }

        assert!(called1);
        assert!(called2);
        assert_eq!(some_buffer, vec![42]);
    }

    #[test]
    fn test_unsubscribe() {
        let mut called = 0u8;
        {
            let mut my_event = MutEvent::<()>::new();
            let subscription = my_event.subscribe(|_| {
                called += 1;
            });
            my_event.emit(());
            my_event.emit(());
            my_event.unsubscribe(subscription).unwrap();
            my_event.emit(());
            my_event.emit(());
        }
        assert_eq!(called, 2);
    }
}
