# Reactivity

A lightweight library for reactive programming with signals in Rust.

[![Crates.io](https://img.shields.io/crates/v/reactivity.svg)](https://crates.io/crates/reactivity)
[![Documentation](https://docs.rs/reactivity/badge.svg)](https://docs.rs/reactivity)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/reactivity.svg)](README.md#license)

## Overview

Reactivity is a Rust library that provides a flexible reactive programming system. It allows you to create signals that can depend on other signals, with automatic propagation of changes through your dependency graph.

## Features

- Two signal implementations:
  - `reactivity::Signal` for single-threaded contexts
  - `reactivity::sync::Signal` for thread-safe, multi-threaded contexts
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

### Single-Threaded Signals

```rust
use reactivity::Signal;
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

### Thread-Safe Signals

```rust
use std::thread;
use reactivity::sync::Signal;
use reactivity::signal;

fn main() {
    // Create a thread-safe signal
    let count = signal!(0);
    
    // Create dependent signals
    let doubled = signal!([count] count * 2);
    
    // Clone for use in another thread
    let count_clone = count.clone();
    
    // Spawn a thread that updates the signal
    let handle = thread::spawn(move || {
        for i in 1..=5 {
            count_clone.send(i);
            thread::sleep(std::time::Duration::from_millis(100));
        }
    });
    
    // Wait for thread to complete
    handle.join().unwrap();
    
    // Main thread can access the updated value
    assert_eq!(doubled.get(), 10);
}
```

## API Overview

### Signal Types

- `Signal<T>`: For single-threaded contexts (uses `Rc` and `RefCell` internally)
- `sync::Signal<T>`: Thread-safe implementation (uses `Arc` and `RwLock` internally)

```rust
// Single-threaded signal
use reactivity::Signal;
let x = Signal::new(42);

// Thread-safe signal
use reactivity::sync::Signal;
let y = Signal::new(42);

// Create dependent signals
let a = Signal::new(1);
let b = signal!([a] a * 2);

// Register dependency (b will update when a changes)
a.add_receiver(b);
```

### signal! Macro

The `signal!` macro provides a convenient way to create signals. The macro automatically uses the appropriate Signal type based on the context:

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

### Choosing Between Signal Types

- Use `reactivity::Signal` for single-threaded applications where all signals are accessed from the same thread
- Use `reactivity::sync::Signal` when signals need to be shared across multiple threads

## License

This project is dual-licensed under either of:

* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you shall be dual licensed as above, without any additional terms or conditions.
