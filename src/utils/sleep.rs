use futures::channel::oneshot;

/// Debug-only delay function to avoid adding another dependency just for delay
#[cfg(debug_assertions)]
pub async fn async_delay(duration: std::time::Duration) -> Result<(), ()> {
    let (tx, rx) = oneshot::channel();

    std::thread::spawn(move || {
        std::thread::sleep(duration);
        //TODO: treat error?
        tx.send(()).map_err(|_| ())
    });

    rx.await.unwrap();
    Ok(())
}

/// Debug-only delay function to avoid adding another dependency just for delay
#[cfg(debug_assertions)]
pub async fn sleep_forever() -> Result<(), ()> {
    while let Ok(_) = async_delay(std::time::Duration::from_secs(1)).await {}
    Err(())
}
