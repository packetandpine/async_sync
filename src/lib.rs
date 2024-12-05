use tokio::task;
use log::{error, info, Level};
use tokio::time::{self, sleep, Duration};
use thiserror::Error;
use futures::future;
use async_trait::async_trait;
use std::time::Instant;

#[derive(Debug, Error)]
pub enum AsyncSyncError {
    #[error("Execution error: {0}")]
    ExecutionError(String),
    #[error("Execution timed out")]
    Timeout,
}

pub enum Backoff {
    Exponential,
    Constant,
}

pub enum Runtime {
    Tokio,
    AsyncStd,
}

pub async fn sync_to_async_with_retries<F, T, E>(
    func: F,
    retries: usize,
    delay: Duration,
    backoff: Backoff,
) -> Result<T, E>
where
    F: Fn() -> Result<T, E> + Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
{
    let mut attempt = 0;
    loop {
        match func() {
            Ok(result) => return Ok(result),
            Err(_err) if attempt < retries => {
                attempt += 1;
                match backoff {
                    Backoff::Constant => sleep(delay).await,
                    Backoff::Exponential => sleep(delay * 2u32.pow(attempt as u32)).await,
                }
            }
            Err(err) => return Err(err),
        }
    }
}
pub async fn sync_to_async_with_logging<F, T>(
    func: F,
    log_message: &str,
    log_level: Level,
) -> Result<T, AsyncSyncError>
where
    F: Fn() -> T + Send + 'static,
    T: Send + 'static,
{
    let result = task::spawn_blocking(func).await;
    if let Err(e) = &result {
        match log_level {
            Level::Error => error!("{}: {:?}", log_message, e),
            _ => {}
        }
    }
    result.map_err(|e| AsyncSyncError::ExecutionError(format!("{:?}", e)))
}

pub async fn parallel_sync_to_async<F, T>(tasks: Vec<F>) -> Vec<Result<T, AsyncSyncError>>
where
    F: Fn() -> T + Send + 'static,
    T: Send + 'static,
{
    let futures = tasks.into_iter().map(|task| task::spawn_blocking(task));
    let results = future::join_all(futures).await;
    results
        .into_iter()
        .map(|res| res.map_err(|e| AsyncSyncError::ExecutionError(format!("{:?}", e))))
        .collect()
}

pub async fn sync_to_async_with_args<F, T, Args>(
    func: F,
    args: Args,
) -> Result<T, AsyncSyncError>
where
    F: Fn(Args) -> T + Send + 'static,
    Args: Send + 'static,
    T: Send + 'static,
{
    task::spawn_blocking(move || func(args))
        .await
        .map_err(|e| AsyncSyncError::ExecutionError(format!("{:?}", e)))
}

#[async_trait]
pub trait AsyncTask {
    async fn run_task(&self) -> Result<(), AsyncSyncError>;
}

pub struct TaskRunner;

#[async_trait]
impl AsyncTask for TaskRunner {
    async fn run_task(&self) -> Result<(), AsyncSyncError> {
        Ok(())
    }
}

pub async fn sync_to_async_with_timeout_and_cleanup<F, T, C>(
    func: F,
    timeout: Duration,
    cleanup: C,
) -> Result<T, AsyncSyncError>
where
    F: Fn() -> T + Send + 'static,
    C: Fn() + Send + 'static,
    T: Send + 'static,
{
    let handle = task::spawn_blocking(func);
    tokio::select! {
        result = handle => result.map_err(|_| AsyncSyncError::ExecutionError("Error executing".into())),
        _ = time::sleep(timeout) => {
            cleanup();
            Err(AsyncSyncError::Timeout)
        }
    }
}

pub async fn sync_to_async_with_diagnostics<F, T>(func: F) -> Result<T, AsyncSyncError>
where
    F: Fn() -> T + Send + 'static,
    T: Send + 'static,
{
    let start = Instant::now();
    let result = task::spawn_blocking(func).await;
    let duration = start.elapsed();
    info!("Execution time: {:?}", duration);
    result.map_err(|e| AsyncSyncError::ExecutionError(format!("{:?}", e)))
}

pub async fn aggregate_results<F, T>(tasks: Vec<F>) -> Vec<Result<T, AsyncSyncError>>
where
    F: Fn() -> T + Send + 'static,
    T: Send + 'static,
{
    let futures = tasks.into_iter().map(|task| task::spawn_blocking(task));
    let results = future::join_all(futures).await;
    results
        .into_iter()
        .map(|res| res.map_err(|e| AsyncSyncError::ExecutionError(format!("{:?}", e))))
        .collect()
}

pub fn sync_to_async_with_runtime<F, T>(runtime: Runtime, func: F) -> Result<T, AsyncSyncError>
where
    F: Fn() -> T + Send + 'static,
    T: Send + 'static,
{
    match runtime {
        Runtime::Tokio => Ok(tokio::runtime::Runtime::new().unwrap().block_on(async { func() })),
        Runtime::AsyncStd => Ok(async_std::task::block_on(async { func() })),
    }
}

/// Macro for sync-to-async conversion
#[macro_export]
macro_rules! sync_to_async {
    ($fn_expr:expr) => {
        async move {
            Ok(task::spawn_blocking(move || $fn_expr())
                .await
                .map_err(|e| AsyncSyncError::ExecutionError(format!("{:?}", e)))?)
        }
    };
}

/// Runs an async function in a synchronous context.
pub fn async_to_sync<F, T>(future: F) -> Result<T, AsyncSyncError>
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    Ok(tokio::runtime::Runtime::new()
        .expect("Failed to create Tokio runtime")
        .block_on(future))
}

/// Converts a synchronous function to async, with an optional timeout.
pub async fn sync_to_async_with_timeout<F, T>(func: F, timeout: Duration) -> Result<T, AsyncSyncError>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    match time::timeout(timeout, task::spawn_blocking(func)).await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(e)) => Err(AsyncSyncError::ExecutionError(format!("{:?}", e))),
        Err(_) => Err(AsyncSyncError::Timeout),
    }
}
