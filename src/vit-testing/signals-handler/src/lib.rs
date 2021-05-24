use futures::future::{FusedFuture, FutureExt};

pub async fn with_signal_handler<F, E>(mut f: F) -> Result<(), E>
where
    F: FusedFuture<Output = Result<(), E>> + Unpin,
{
    futures::select! {
        result = f => result,
        _ = watch_signal().fuse() => Ok(()),
    }
}

#[cfg(unix)]
async fn watch_signal() {
    use tokio::signal::unix::{signal, SignalKind};

    let mut hsigterm =
        signal(SignalKind::interrupt()).expect("failed to install handler for SIGTERM");
    let mut hsigint =
        signal(SignalKind::terminate()).expect("failed to install handler for SIGTINT");

    let hsigterm_future = hsigterm.recv();
    let hsigint_future = hsigint.recv();

    tokio::pin!(hsigterm_future);
    tokio::pin!(hsigint_future);

    futures::future::select(hsigterm_future, hsigint_future).await;
    eprintln!("stopped upon receiving SIGTERM or SIGINT");
}

#[cfg(not(unix))]
async fn watch_signal() {
    use tokio::signal::ctrl_c;

    ctrl_c().await.expect("failed to wait for Ctrl-C");
    eprintln!("stopped by an external signal");
}
