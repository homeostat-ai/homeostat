use std::error::Error;

use crate::{banner, observability};

pub async fn start() -> Result<(), Box<dyn Error + Send + Sync>> {
    banner::print("controller");
    observability::serve(shutdown_signal()).await
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            eprintln!("failed to listen for Ctrl+C: {error}");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => eprintln!("failed to listen for SIGTERM: {error}"),
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    println!("shutting down controller");
}
