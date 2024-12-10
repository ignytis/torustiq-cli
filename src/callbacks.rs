/// These functions are passed to modules and called from there
/// It's preferred way to handle the data asynchronously using data and
/// system message channels in order not to block the module routines

use log::{debug, error};

use torustiq_common::ffi::types::module::{ModuleHandle, Record};

use crate::xthread::{FREE_BUF, SENDERS, SYSTEM_MESSAGES, SystemMessage};

/// Called from modules on step thread termination
pub extern "C"  fn on_step_terminate_cb(module_handle: ModuleHandle) {
    debug!("A step termination signal is triggered from step with index {}", module_handle);
    let msg_chan = match SYSTEM_MESSAGES.get() {
        Some(c) => c,
        None => {
            error!("Termination callback failure: the system message channel is not initialized.");
            return
        }
    };
    if let Err(e) = msg_chan.send(SystemMessage::TerminateStep(module_handle)) {
        error!("Termination callback failure: cannot send a termination message ({})", e);
    }
}

/// Steps use this function to pass the produced record to dependent step
pub extern "C" fn on_rcv_cb(record: Record, module_handle: ModuleHandle) {
    let sender = match SENDERS.lock().unwrap().get(&module_handle) {
        Some(s) => s.clone(),
        None => return, // no sender exists: no action
    };

    // Sends a cloned record to further processing and deallocates the original record
    if let Err(e) = sender.send(record.clone()) {
        error!("Failed to send a record from step '{}' to the next steep: {}", module_handle, e);
        return;
    }
    let free_buf_fn_map = FREE_BUF.lock().unwrap();
    let free_buf_fn = free_buf_fn_map.get(&module_handle).unwrap();
    free_buf_fn(record);
}