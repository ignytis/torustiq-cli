use std::sync::atomic::{AtomicBool, Ordering};
use log::{error, info};

use crate::xthread::PIPELINE;

static mut TODO_TERMINATE: AtomicBool = AtomicBool::new(false);

/// Initializes a system signal handler (e.g. handles CTRL+C)
pub fn init_signal_handler() -> Result<(), String> {
    match ctrlc::set_handler(|| {
        info!("Received a termination signal in main thread");

        let pipeline = unsafe {
            match PIPELINE.get() {
                Some(p) => p,
                None => {
                    error!("Cannot receive a pipeline singleton");
                    return;
                }
            }
        };

        let pipeline = pipeline.lock().unwrap();
        pipeline.trigger_termination();
    }) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to init a signal handler: {}", e)),
    }
}

pub fn is_termination_requested() -> bool {
    unsafe { TODO_TERMINATE.load(Ordering::SeqCst) }
}