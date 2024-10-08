/// These functions are passed to modules and called from there
/// It's preferred way to handle the data asynchronously using data and
/// system message channels in order not to block the module routines

use log::{debug, error};

use torustiq_common::ffi::types::module::{ModuleStepHandle, Record};

use crate::xthread::{FREE_BUF, SENDERS, SYSTEM_MESSAGES, SystemMessage};

/// Called from modules on step thread termination
pub extern "C"  fn on_step_terminate_cb(step_handle: ModuleStepHandle) {
    debug!("A step termination signal is triggered from step with index {}", step_handle);
    let msg_chan = match SYSTEM_MESSAGES.get() {
        Some(c) => c,
        None => {
            error!("Termination callback failure: the system message channel is not initialized.");
            return
        }
    };
    match msg_chan.send(SystemMessage::TerminateStep(step_handle)) {
        Ok(_) => {},
        Err(e) => error!("Termination callback failure: cannot send a termination message ({})", e)
    };
}

/// Steps use this function to pass the produced record to dependent step
pub extern "C" fn on_rcv_cb(record: Record, step_handle: ModuleStepHandle) {
    let sender = match SENDERS.lock().unwrap().get(&step_handle) {
        Some(s) => s.clone(),
        None => return, // no sender exists: no action
    };

    // Sends a cloned record to further processing and deallocates the original record
    match sender.send(record.clone()) {
        Ok(_) => {},
        Err(e) => {
            error!("Failed to send a record from step '{}' to the next steep: {}", step_handle, e);
            return;
        }
    };
    let free_buf_fn_map = FREE_BUF.lock().unwrap();
    let free_buf_fn = free_buf_fn_map.get(&step_handle).unwrap();
    free_buf_fn(record);
}