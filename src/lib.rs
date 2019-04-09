//! This crate provides implementation of [observer](https://en.wikipedia.org/wiki/Observer_pattern) design pattern.
//!
//! Events are strongly typed, with immediate dispatch and support subscription and unsubscription
//! of multiple handlers.
//!
//! Events are defined using `event!` macro. It allows user to fully control lifetime, mutability
//! and thread safety constraints for event handlers.
//!
//! # Syntax
//!
//! ```ignore
//! event!(
//!     /// Optional doc comments
//!     EventName[<'lifetime>] => [Fn|FnMut]([arg_name: ArgType, ...]) [+ Send + Sync + 'lifetime]
//! );
//! ```
//!
//! Event arguments must be `Clone`able types.
//!
//! # Examples
//!
//! ```
//! use eventd::event;
//!
//! // Event handlers are static immutable and not thread-safe
//! event!(Event1 => Fn(x: u32) + 'static);
//!
//! // Event handlers are with lifetime `'a` mutable and not thread-safe
//! event!(Event2<'a> => FnMut(y: u32) + 'a);
//!
//! // Event handlers are static, thread-safe and immutable
//! event!(Event3 => Fn(z: u32) + Send + Sync + 'static);
//! ```
//!
//! # Example usage
//!
//! ```rust
//! #[macro_use]
//! extern crate eventd;
//!
//! event!(MyEvent => Fn(x: u8) + 'static);
//!
//! fn main() {
//!     let mut my_event = MyEvent::default();
//!     let _ = my_event.subscribe(|x| println!("Got {}", x));
//!     my_event.emit(42);
//! }
//! ```
use std::fmt;

#[doc(hidden)]
pub use slab::Slab;


/// Token used to differentiate subscribers.
///
/// Should be passed back to event dispatcher to unsubscribe.
pub struct Subscription {
    #[doc(hidden)]
    pub key: usize,
}


/// Error emmited on attempt to unsubscribe with invalid subscription.
#[derive(Debug)]
pub struct SubscriptionMissing;

impl fmt::Display for SubscriptionMissing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Attempt to unsubscribe delegate without subscription")
    }
}

impl std::error::Error for SubscriptionMissing {}


/// Macro used for defining event dispatcher types.
///
/// See crate documentation for usage.
#[macro_export(local_inner_macros)]
macro_rules! event {
    (
        $(#[$attr:meta])*
        $name:ident
        $(< $lt:lifetime >)? => Fn($($arg_name:ident : $arg_ty:ty),*)
        $(+ $bound:tt)*
    ) => {
        __event_impl!(
            $(#[$attr])*,
            $name$(< $lt >)? => Fn,
            [$($arg_name: $arg_ty),*],
            [$($bound),*],
            self: &Self, &self.handlers
        );
    };
    (
        $(#[$attr:meta])*
        $name:ident
        $(< $lt:lifetime >)? => FnMut($($arg_name:ident : $arg_ty:ty),*)
        $(+ $bound:tt)*
    ) => {
        __event_impl!(
            $(#[$attr])*,
            $name$(< $lt >)? => FnMut,
            [$($arg_name: $arg_ty),*],
            [$($bound),*],
            self: &mut Self, &mut self.handlers
        );
    };
}


#[doc(hidden)]
#[macro_export]
macro_rules! __event_impl {
    (
        $(#[$attr:meta])*,
        $name:ident $(< $lt:lifetime >)? => $fn:tt,
        [$($arg_name:ident: $arg_ty:ty),*],
        [$($bound:tt),*],
        $self:ident: $self_ty:ty, $iter_ex:expr
    ) => {
        $(#[$attr])*
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
            /// Subscribes a closure to be called on event emmision.
            ///
            /// Return subscription token.
            pub fn subscribe<F>(&mut self, handler: F) -> $crate::Subscription
            where
                F: $fn($($arg_ty),*) $( + $bound)*,
            {
                $crate::Subscription {
                    key: self.handlers.insert(Box::new(handler)),
                }
            }

            /// Unsubscribes handler for given subscription token.
            ///
            /// Returns error if there is no handler for given subscription.
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

            /// Dispatches a call with given arguments to all subscribed handlers.
            ///
            /// Arguments must be clonable.
            pub fn emit($self:$self_ty, $($arg_name: $arg_ty),*) {
                for (_, handler) in $iter_ex {
                    (*handler)($($arg_name.clone()),*)
                }
            }
        }
    };
}

pub mod example {
    //! Module with sample event.
    //!
    //! Generated using:
    //!
    //! ```
    //! use eventd::event;
    //!
    //! event!(
    //!     /// Sample event generated using `event!` macro.
    //!     ExampleEvent<'a> => Fn(x: u32, y: &str) + Sync + 'a
    //! );
    //! ```
    event!(
        /// Sample event generated using `event!` macro.
        ExampleEvent<'a> => Fn(x: u32, y: &str) + Sync + 'a
    );
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
