# async_sync

Effortlessly integrate synchronous and asynchronous code in Rust.  
The `async_sync` crate simplifies the boundaries between sync and async workflows, providing tools for retry mechanisms, backoff strategies, timeouts, diagnostics, and parallel execution of sync tasks.

## Features

- **Retry Mechanisms**: With configurable backoff strategies (constant, linear, exponential).
- **Timeout Handling**: Include optional cleanup logic on timeout.
- **Parallel Execution**: Run multiple sync tasks concurrently.
- **Integration**: Seamlessly works with `Tokio` and `Async-std` runtimes.
- **Diagnostics**: Log execution times and track performance.

---

## Getting Started

Add `async_sync` to your `Cargo.toml`:

```toml
[dependencies]
async_sync = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

## Examples

### Retry with Backoff Strategy

Handle transient failures in sync tasks with retries and exponential backoff.

```rust
use async_sync::{sync_to_async_with_retries, Backoff};
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn flaky_function(counter: Arc<Mutex<i32>>) -> Result<String, &'static str> {
    let mut count = counter.lock().unwrap();
    *count += 1;
    if *count < 3 {
        Err("Failure")
    } else {
        Ok("Success")
    }
}

#[tokio::main]
async fn main() {
    let counter = Arc::new(Mutex::new(0));
    let result = sync_to_async_with_retries(
        || flaky_function(Arc::clone(&counter)),
        5,                          // Max retries
        Duration::from_millis(100), // Initial delay
        Backoff::Exponential,       // Backoff strategy
    )
    .await;

    match result {
        Ok(value) => println!("Task succeeded: {}", value),
        Err(err) => eprintln!("Task failed: {:?}", err),
    }
}
```

## Examples

### Retry with Backoff Strategy

Handle transient failures in sync tasks with retries and exponential backoff.

```rust
use async_sync::{sync_to_async_with_retries, Backoff};
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn flaky_function(counter: Arc<Mutex<i32>>) -> Result<String, &'static str> {
    let mut count = counter.lock().unwrap();
    *count += 1;
    if *count < 3 {
        Err("Failure")
    } else {
        Ok("Success")
    }
}

#[tokio::main]
async fn main() {
    let counter = Arc::new(Mutex::new(0));
    let result = sync_to_async_with_retries(
        || flaky_function(Arc::clone(&counter)),
        5,                          // Max retries
        Duration::from_millis(100), // Initial delay
        Backoff::Exponential,       // Backoff strategy
    )
    .await;

    match result {
        Ok(value) => println!("Task succeeded: {}", value),
        Err(err) => eprintln!("Task failed: {:?}", err),
    }
}
```

### Parallel Execution of Sync Tasks

Execute multiple synchronous tasks in parallel and collect their results.

```rust
use async_sync::parallel_sync_to_async;

fn heavy_computation(x: i32) -> i32 {
    std::thread::sleep(std::time::Duration::from_millis(200));
    x * x
}

#[tokio::main]
async fn main() {
    let tasks: Vec<_> = (1..=5).map(|x| move || heavy_computation(x)).collect();
    let results = parallel_sync_to_async(tasks).await;

    for (i, result) in results.into_iter().enumerate() {
        println!("Task {} result: {:?}", i + 1, result.unwrap());
    }
}
```

### Sync-Async Integration with Runtimes

Run sync functions using the `Tokio` or `Async-std` runtime.

```rust
use async_sync::{sync_to_async_with_runtime, Runtime};

fn heavy_computation(x: i32) -> i32 {
    std::thread::sleep(std::time::Duration::from_millis(200));
    x * x
}

fn main() {
    let result = sync_to_async_with_runtime(Runtime::Tokio, || heavy_computation(4));
    println!("Result: {:?}", result.unwrap());
}
```

---

## Contact

If you have any questions, feedback, or just want to connect, feel free to reach out!

- **Matthew Novak**  
  Email: [matt@packetandpine.com](mailto:matt@packetandpine.com)  
  Website: [https://packetandpine.com](https://packetandpine.com)  
  Twitter: [https://x.com/PacketAndPine](https://x.com/PacketAndPine)  
  YouTube: [https://www.youtube.com/@PacketAndPine](https://www.youtube.com/@PacketAndPine)

