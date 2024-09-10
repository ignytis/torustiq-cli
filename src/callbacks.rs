/// These functions are passed to modules and called from there

use log::info;

use crate::{
    shutdown::todo_terminate,
    xthread::{FREE_BUF, SENDERS}
};
use torustiq_common::ffi::types::{
    module::{ModuleStepHandle, Record},
    std_types,
};

/// Called from modules to trigger shutdown of the app
pub extern "C" fn on_terminate_cb(step_handle: std_types::Uint) {
    info!("Received a termination signal from step with index {}", step_handle);
    todo_terminate();
}

/// Steps use this function to pass the produced record to dependent step
pub extern "C" fn on_rcv_cb(record: Record, step_handle: ModuleStepHandle) {
    let sender = match SENDERS.lock().unwrap().get(&step_handle) {
        Some(s) => s.clone(),
        None => return, // no sender exists: no action
    };

    // Sends a cloned record to further processing and deallocates the original record
    sender.send(record.clone()).unwrap();
    let free_buf_fn_map = FREE_BUF.lock().unwrap();
    let free_buf_fn = free_buf_fn_map.get(&step_handle).unwrap();
    free_buf_fn(record);
}