/// These functions are passed to modules and called from there

use log::{debug, error, info};

use crate::{
    pipeline, shutdown::set_termination_flag, xthread::{FREE_BUF, PIPELINE, SENDERS}
};
use torustiq_common::ffi::types::{
    module::{ModuleStepHandle, Record},
    std_types,
};

/// Called from modules on step thread termination
pub extern "C" fn on_step_terminate_cb(step_handle: std_types::Uint) {
    debug!("Received a termination signal from step with index {}", step_handle);
    let pipeline = unsafe {
        match PIPELINE.get_mut() {
            Some(p) => p.lock().unwrap(),
            None => {
                error!("Cannot process the retmination callback for step {}: \
                    pipeline is not registered in static context", step_handle);
                return;
            }
        }
    };
    let pipeline_step_arc = match pipeline.get_step_by_handle_mut(usize::try_from(step_handle).unwrap()) {
        Some(s) => s,
        None => {
            error!("Cannot find a pipeline step with handle '{}' in static context", step_handle);
            return;
        }
    };
    let mut pipeline_step = pipeline_step_arc.lock().unwrap();
    pipeline_step.set_state_terminated();
    // set_termination_flag();
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