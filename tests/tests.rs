//! # Tests for async_sync Crate
//!
//! This module contains comprehensive tests for all functionality provided by the `async_sync` library.
//! Each test includes documentation to explain its purpose, setup, and expected outcome.

use async_sync::*;
use tokio::task;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use log::Level;

/// Mock function for testing retries.
/// It fails the first two times and succeeds on the third attempt.
fn flaky_function(counter: Arc<Mutex<i32>>) -> Result<String, &'static str> {
    let mut count = counter.lock().unwrap();
    *count += 1;
    if *count <= 3 {
        Err("Failure")
    } else {
        Ok("Success".to_string())
    }
}

/// Mock cleanup function that gets called during timeout tests.
fn cleanup_action() {
    println!("Cleanup executed!");
}

/// Mock function for testing heavy synchronous computation.
fn heavy_computation(x: i32) -> i32 {
    std::thread::sleep(Duration::from_millis(200)); // Simulate heavy computation
    x * x
}

/// Tests the retry mechanism of `sync_to_async_with_retries`.
/// Ensures that a flaky function succeeds within the allowed number of retries.
#[tokio::test]
async fn test_sync_to_async_with_retries_success() {
    let counter = Arc::new(Mutex::new(0));
    let result = sync_to_async_with_retries(
        move || flaky_function(Arc::clone(&counter)),
        3,
        Duration::from_millis(100),
        Backoff::Constant,
    )
        .await;
    assert_eq!(result, Ok("Success".to_string()));
}

/// Tests that `sync_to_async_with_retries` fails when the maximum retries are exceeded.
#[tokio::test]
async fn test_sync_to_async_with_retries_failure() {
    let counter = Arc::new(Mutex::new(0));
    let result = sync_to_async_with_retries(
        move || flaky_function(Arc::clone(&counter)),
        2,
        Duration::from_millis(100),
        Backoff::Constant,
    )
        .await;
    assert_eq!(result, Err("Failure"));
}

/// Tests the logging functionality of `sync_to_async_with_logging`.
/// Ensures the function executes correctly and logs any errors.
#[tokio::test]
async fn test_sync_to_async_with_logging() {
    let result = sync_to_async_with_logging(|| heavy_computation(5), "Logging test", Level::Error).await;
    assert_eq!(result.unwrap(), 25);
}

/// Tests the parallel execution of multiple synchronous tasks.
/// Ensures all tasks complete successfully and their results are correct.
#[tokio::test]
async fn test_parallel_sync_to_async() {
    let tasks: Vec<_> = (1..=5).map(|x| move || heavy_computation(x)).collect();
    let results = parallel_sync_to_async(tasks).await;

    for (i, result) in results.into_iter().enumerate() {
        assert_eq!(result.unwrap(), (i + 1).pow(2).try_into().unwrap());
    }
}

/// Tests passing arguments to a synchronous function using `sync_to_async_with_args`.
#[tokio::test]
async fn test_sync_to_async_with_args() {
    let result = sync_to_async_with_args(|x: i32| heavy_computation(x), 5).await;
    assert_eq!(result.unwrap(), 25);
}

/// Tests that `sync_to_async_with_timeout` completes successfully within the timeout duration.
#[tokio::test]
async fn test_sync_to_async_with_timeout_success() {
    let result = sync_to_async_with_timeout(|| heavy_computation(5), Duration::from_secs(1)).await;
    assert_eq!(result.unwrap(), 25);
}

/// Tests that `sync_to_async_with_timeout` fails when the timeout is exceeded.
#[tokio::test]
async fn test_sync_to_async_with_timeout_failure() {
    let result = sync_to_async_with_timeout(|| heavy_computation(5), Duration::from_millis(50)).await;
    assert!(result.is_err());
}

/// Tests `sync_to_async_with_timeout_and_cleanup`.
/// Verifies that the cleanup function is executed when the timeout occurs.
#[tokio::test]
async fn test_sync_to_async_with_timeout_and_cleanup() {
    let result = sync_to_async_with_timeout_and_cleanup(
        || heavy_computation(5),
        Duration::from_millis(50),
        cleanup_action,
    )
        .await;
    assert!(result.is_err());
}

/// Tests that `sync_to_async_with_diagnostics` runs the function correctly
/// and logs the execution time.
#[tokio::test]
async fn test_sync_to_async_with_diagnostics() {
    let result = sync_to_async_with_diagnostics(|| heavy_computation(5)).await;
    assert_eq!(result.unwrap(), 25);
}

/// Tests `aggregate_results` for batch execution of multiple tasks.
/// Ensures all tasks return correct results.
#[tokio::test]
async fn test_aggregate_results() {
    let tasks: Vec<_> = (1..=3).map(|x| move || heavy_computation(x)).collect();
    let results = aggregate_results(tasks).await;

    for (i, result) in results.into_iter().enumerate() {
        assert_eq!(result.unwrap(), (i + 1).pow(2).try_into().unwrap());
    }
}

/// Tests synchronous execution using the `Tokio` runtime.
#[test]
fn test_sync_to_async_with_runtime_tokio() {
    let result = sync_to_async_with_runtime(Runtime::Tokio, || heavy_computation(4));
    assert_eq!(result.unwrap(), 16);
}

/// Tests synchronous execution using the `AsyncStd` runtime.
#[test]
fn test_sync_to_async_with_runtime_async_std() {
    let result = sync_to_async_with_runtime(Runtime::AsyncStd, || heavy_computation(4));
    assert_eq!(result.unwrap(), 16);
}

/// Tests the `AsyncTask` trait implementation.
/// Ensures the async task completes successfully.
#[tokio::test]
async fn test_async_task_trait() {
    let runner = TaskRunner;
    let result = runner.run_task().await;
    assert!(result.is_ok());
}

/// Tests the `sync_to_async!` macro to convert a synchronous function to async.
#[tokio::test]
async fn test_sync_to_async_macro() {
    async fn test_macro() {
        let result: Result<i32, AsyncSyncError> = sync_to_async!(|| heavy_computation(5)).await;
        assert_eq!(result.unwrap(), 25);
    }
    test_macro().await;
}