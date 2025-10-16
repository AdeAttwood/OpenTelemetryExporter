use std::sync::{Arc, atomic::AtomicBool};

mod config;
mod exporters;
mod log;
mod otlp_init;
mod run;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    {
        if std::env::args().any(|arg| arg == "--service") {
            // Windows: run as service
            return run::windows_service_support::run_as_service().map_err(|e| e.into());
        }
    }

    // All platforms: run normally
    let running = Arc::new(AtomicBool::new(true));
    run::run_app(running).await
}
