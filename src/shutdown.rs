use std::{process::exit, sync::atomic::{AtomicBool, Ordering}};

use log::{error, info};

use crate::xthread::PIPELINE;

static IS_GRACEFUL_SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Initializes a system signal handler (e.g. handles CTRL+C)
pub fn init_signal_handler() -> Result<(), String> {
    match ctrlc::set_handler(|| {
        info!("Received a termination signal in main thread");
        match IS_GRACEFUL_SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
            false => { // Signal received once: shutdown gracefully
                info!("Shutting down gracefully...");
                IS_GRACEFUL_SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);

                let pipeline = match PIPELINE.get() {
                    Some(p) => p,
                    None => {
                        error!("Cannot receive a pipeline singleton");
                        return;
                    }
                };

                let pipeline = pipeline.lock().unwrap();
                pipeline.trigger_termination();
            },
            true => { // force shutdown
                info!("A graceful shutdown had been requested already. Shutting down forcefully...");
                exit(-1)
            }
        };
    }) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to init a signal handler: {}", e)),
    }
}
