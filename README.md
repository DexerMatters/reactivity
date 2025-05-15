# Reactivity

A lightweight library for reactive programming with signals in Rust.

[![Crates.io](https://img.shields.io/crates/v/reactivity.svg)](https://crates.io/crates/reactivity)
[![Documentation](https://docs.rs/reactivity/badge.svg)](https://docs.rs/reactivity)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/reactivity.svg)](README.md#license)

## Overview

Reactivity is a Rust library that provides a flexible reactive programming system. It allows you to create signals that can depend on other signals, with automatic propagation of changes through your dependency graph.

## Features

- Thread-safe reactive signals
- Clean API for creating and managing signals
- Convenient macro for defining reactive computations
- Support for side effects when signals change
- Fine-grained control over reaction propagation

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
reactivity = "0.1.0"
```

## Basic Usage

```rust
use reactivity::signal;

fn main() {
    // Create a basic signal
    let count = signal!(0);
    
    // Create a derived signal that depends on count
    let doubled = signal!([count] count * 2);
    
    // Create another signal with a side effect
    let message = signal!(<old_val, new_val> [count] 
        format!("The count is {}", count); 
        println!("Count changed from {} to {}!", old_val, new_val)
    );
    
    // Update the original signal
    count.send(5);
    
    // The changes have been automatically propagated
    assert_eq!(count.get(), 5);
    assert_eq!(doubled.get(), 10);
    assert_eq!(message.get(), "The count is 5");
}
```

## API Overview

### Signal

`Signal<T>` is the main type for reactive values. It can:

- Hold a value that can be read with `get()` or `borrow()`
- Be updated with new values via `send()`
- Depend on other signals and react to their changes
- Have other signals depend on it

```rust
// Create an independent signal
let x = Signal::new(42);

// Read the value
let value = x.get();

// Update the value
x.send(100);
```

### signal! Macro

The `signal!` macro provides a convenient way to create signals:

```rust
// Basic signal with initial value
let x = signal!(5);

// Signal that depends on other signals
let y = signal!([x] x + 10);

// Signal with side effect
let z = signal!(<old_val, new_val> [x, y] {
    let sum = x + y;
    sum * 2
}; println!("z changed from {} to {}", old_val, new_val));
```

## Advanced Usage

### Custom Effect Functions

You can specify custom effects that run when signals change:

```rust
let x = signal!(1);
let y = signal!(<old_y, new_y> [x] x * 3; {
    println!("y changed from {} to {}", old_y, new_y);
    // Perform side effects here
});
```

### Thread-Safe Signals

The default `Signal<T>` is thread-safe, using `Arc` and `parking_lot::RwLock` internally.

## License

This project is dual-licensed under either of:

* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you shall be dual licensed as above, without any additional terms or conditions.
