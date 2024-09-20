use std::future::Future;

/// Run the test in another thread while sending txs to force binance to mine new blocks
/// # Panic
/// Panics if the future panics
pub async fn run_test<Fut: Future<Output = ()> + Send + 'static>(future: Fut) {
    // Guarantee that only one test is incrementing blocks at a time
    static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    // Run the test in another thread
    let test_handler = tokio::spawn(future);

    // Acquire Lock
    let guard = LOCK.lock().await;

    // Check if the test is finished after acquiring the lock
    if test_handler.is_finished() {
        // Release lock
        drop(guard);

        // Now is safe to panic
        if let Err(err) = test_handler.await {
            std::panic::resume_unwind(err.into_panic());
        }
        return;
    }

    // Now is safe to panic
    if let Err(err) = test_handler.await {
        // Resume the panic on the main task
        std::panic::resume_unwind(err.into_panic());
    }
}
