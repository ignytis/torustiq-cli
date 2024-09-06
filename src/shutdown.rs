use std::sync::atomic::{AtomicBool, Ordering};
use log::info;

use torustiq_common::ffi::types::std_types;

static mut TODO_TERMINATE: AtomicBool = AtomicBool::new(false);

fn todo_terminate() {
    unsafe {
        TODO_TERMINATE.store(true, Ordering::SeqCst);
    }
}

/// Called from modules to trigger shutdown of the app
pub extern "C" fn on_terminate_cb(step_handle: std_types::Uint) {
    info!("Received a termination signal from step with index {}", step_handle);
    todo_terminate();
}

pub fn init_signal_handler() {
    ctrlc::set_handler(|| {
        info!("Received a termination signal in main thread");
        todo_terminate();
    }).expect("Could not send signal on channel.");
}

pub fn is_termination_requested() -> bool {
    unsafe { TODO_TERMINATE.load(Ordering::SeqCst) }
}