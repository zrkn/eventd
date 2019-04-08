# eventd

Rust implementation of [observer](https://en.wikipedia.org/wiki/Observer_pattern) design pattern.
Dispatch is immediate and multicast. For delayed handling you can use [shrev](https://crates.io/crates/shrev).

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/eventd.svg)](https://crates.io/crates/eventd)
[![Documentation](https://docs.rs/eventd/badge.svg)][dox]

More information about this crate can be found in the [crate documentation][dox].

[dox]: https://docs.rs/eventd/*/eventd/

## Features

* Strongly typed
* Subscribe and unsubscribe of multiple handlers
* Configurable lifetime, mutability and thread safety constraints for handlers

## Usage

To use `eventd`, first add this to your `Cargo.toml`:

```toml
[dependencies]
eventd = "0.3"
```

Next, you can use `event!` macro to define your event signatures and use them:

```rust
#[macro_use]
extern crate eventd;

event!(MyEvent => Fn(x: u8));

fn main() {
    let mut my_event = MyEvent::default();
    my_event.subscribe(|x| println!("Got {}", x));
    my_event.emit(42);
}
```
