use std::sync::atomic::{AtomicBool, Ordering};
use log::info;

static mut TODO_TERMINATE: AtomicBool = AtomicBool::new(false);

pub fn todo_terminate() {
    unsafe {
        TODO_TERMINATE.store(true, Ordering::SeqCst);
    }
}

pub fn init_signal_handler() -> Result<(), String> {
    match ctrlc::set_handler(|| {
        info!("Received a termination signal in main thread");
        todo_terminate();
    }) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to init a signal handler: {}", e)),
    }
}

pub fn is_termination_requested() -> bool {
    unsafe { TODO_TERMINATE.load(Ordering::SeqCst) }
}