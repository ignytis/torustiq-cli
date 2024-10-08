use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use log::{debug, error, info};

use crate::xthread::PIPELINE;

static mut TODO_TERMINATE: AtomicBool = AtomicBool::new(false);


/// Sets the termination flag to true. The main application shuts down when this flag is set
pub fn set_termination_flag() {
    let pipeline = unsafe {
        match PIPELINE.get() {
            Some(p) => p,
            None => {
                error!("Cannot receive a pipeline singleton");
                return;
            }
        }
    };

    thread::spawn(move || {
        debug!("set_termination_flag :: Acquiring lock");
        let pipeline = pipeline.lock().unwrap();
        pipeline.trigger_termination();
        debug!("set_termination_flag :: Releasing lock");
    });
}

/// Initializes a system signal handler (e.g. handles CTRL+C)
pub fn init_signal_handler() -> Result<(), String> {
    match ctrlc::set_handler(|| {
        info!("Received a termination signal in main thread");
        set_termination_flag();
    }) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to init a signal handler: {}", e)),
    }
}

pub fn is_termination_requested() -> bool {
    unsafe { TODO_TERMINATE.load(Ordering::SeqCst) }
}