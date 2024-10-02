/// Cross-thread communication

use std::{
    collections::HashMap,
    sync::{mpsc::Sender, Arc, Mutex},
};

use once_cell::sync::{Lazy, OnceCell};

use torustiq_common::ffi::types::{
    functions::ModuleFreeRecordFn,
    module::{ModuleStepHandle, Record},
};

use crate::pipeline::pipeline::Pipeline;

/// A hashmap of sender channels for each step
/// Senders submit a record to dependent step. currently it's just the next step,
/// but it might change in the future (e.g. multiple steps)
pub static SENDERS: Lazy<Mutex<HashMap<ModuleStepHandle, Sender<Record>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

/// A hashmap of pointers to torustiq_module_free_record functions.
/// As records must be deallocated by modules they are created in, this map allows to
/// locate a deallocation function owned by module
pub static FREE_BUF: Lazy<Mutex<HashMap<ModuleStepHandle, ModuleFreeRecordFn>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});


pub static mut PIPELINE: OnceCell<Arc<Mutex<Pipeline>>> = OnceCell::new();