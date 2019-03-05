use std::fmt;

#[doc(hidden)]
pub use slab::Slab;


pub struct Subscription {
    #[doc(hidden)]
    pub key: usize,
}


#[derive(Debug)]
pub struct SubscriptionMissing;

impl fmt::Display for SubscriptionMissing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Attempt to unsubscribe delegate without subscription")
    }
}

impl std::error::Error for SubscriptionMissing {}


#[macro_export(local_inner_macros)]
macro_rules! event {
    (
        $name:ident
        $(< $lt:lifetime >)? => Fn($($arg_name:ident : $arg_ty:ty),*)
        $(+ $bound:tt)*
    ) => {
        event_impl!($name$(< $lt >)? => Fn, [$($arg_name: $arg_ty),*], [$($bound),*], self: &Self, &self.handlers);
    };
    (
        $name:ident
        $(< $lt:lifetime >)? => FnMut($($arg_name:ident : $arg_ty:ty),*)
        $(+ $bound:tt)*
    ) => {
        event_impl!($name$(< $lt >)? => FnMut, [$($arg_name: $arg_ty),*], [$($bound),*], self: &mut Self, &mut self.handlers);
    };
    (
        $name:ident
        $(< $lt:lifetime >)? => FnOnce($($arg_name:ident : $arg_ty:ty),*)
        $(+ $bound:tt)*
    ) => {
        event_impl!($name$(< $lt >)? => FnOnce, [$($arg_name: $arg_ty),*], [$($bound),*], self: Self, self.handlers);
    };
}


#[doc(hidden)]
#[macro_export]
macro_rules! event_impl {
    ($name:ident $(< $lt:lifetime >)? => $fn:tt, [$($arg_name:ident: $arg_ty:ty),*], [$($bound:tt),*], $self:ident: $self_ty:ty, $iter_ex:expr) => {
        pub struct $name$(<$lt>)? {
            handlers: $crate::Slab<Box<$fn($($arg_ty),*) $( + $bound)*>>,
        }

        impl$(<$lt>)? Default for $name$(<$lt>)? {
            fn default() -> Self {
                $name {
                    handlers: $crate::Slab::new(),
                }
            }
        }

        #[allow(dead_code)]
        impl$(<$lt>)? $name$(<$lt>)?  {
            pub fn subscribe<F>(&mut self, handler: F) -> $crate::Subscription
            where
                F: $fn($($arg_ty),*) $( + $bound)*,
            {
                $crate::Subscription {
                    key: self.handlers.insert(Box::new(handler)),
                }
            }

            pub fn unsubscribe(
                &mut self,
                subscription: $crate::Subscription,
            ) -> Result<(), $crate::SubscriptionMissing> {
                if self.handlers.contains(subscription.key) {
                    self.handlers.remove(subscription.key);
                    Ok(())
                } else {
                    Err($crate::SubscriptionMissing)
                }
            }

            pub fn emit($self:$self_ty, $($arg_name: $arg_ty),*) {
                for (_, handler) in $iter_ex {
                    (*handler)($($arg_name.clone()),*)
                }
            }
        }
    };
}

pub mod example {
    event!(ExampleEvent<'a> => Fn(x: u32, y: &str) + Sync + 'a);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event() {
        event!(MyEvent<'a> => FnMut(x: u8, borrowed: &str) + 'a);

        let mut called1 = false;
        let mut called2 = false;
        let mut some_buffer = Vec::new();
        {
            let mut my_event = MyEvent::default();
            my_event.subscribe(|x, y| {
                called1 = true;
                assert_eq!(x, 42);
                assert_eq!(y, "foo");
            });
            my_event.subscribe(|x, _| {
                called2 = true;
                some_buffer.push(x);
            });

            my_event.emit(42, "foo");
        }

        assert!(called1);
        assert!(called2);
        assert_eq!(some_buffer, vec![42]);
    }

    #[test]
    fn test_unsubscribe() {
        event!(MyEvent<'a> => FnMut() + 'a);

        let mut called = 0u8;
        {
            let mut my_event = MyEvent::default();
            let subscription = my_event.subscribe(|| {
                called += 1;
            });
            my_event.emit();
            my_event.emit();
            my_event.unsubscribe(subscription).unwrap();
            my_event.emit();
            my_event.emit();
        }
        assert_eq!(called, 2);
    }
}
