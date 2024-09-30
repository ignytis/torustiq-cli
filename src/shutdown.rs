use std::sync::atomic::{AtomicBool, Ordering};
use log::info;

static mut TODO_TERMINATE: AtomicBool = AtomicBool::new(false);


/// Sets the termination flag to true. The main application shuts down when this flag is set
pub fn set_termination_flag() {
    unsafe {
        TODO_TERMINATE.store(true, Ordering::SeqCst);
    }
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