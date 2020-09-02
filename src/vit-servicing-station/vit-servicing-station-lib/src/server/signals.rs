use tokio::signal;

#[cfg(not(target_os = "windows"))]
pub async fn watch_signal_for_shutdown() {
    let mut interrupt_stream = signal::unix::signal(signal::unix::SignalKind::interrupt())
        .expect("Error setting up interrupt signal");

    let mut terminate_signal = signal::unix::signal(signal::unix::SignalKind::terminate())
        .expect("Error setting up terminate signal");

    let mut quit_signal = signal::unix::signal(signal::unix::SignalKind::quit())
        .expect("Error setting up quit signal");

    tokio::select! {
        _ = signal::ctrl_c() => (),
        _ = interrupt_stream.recv() => (),
        _ = terminate_signal.recv() => (),
        _ = quit_signal.recv() => (),
    }
}

#[cfg(target_os = "windows")]
pub async fn watch_signal_for_shutdown() {
    signal::ctrl_c().await.ok();
}
