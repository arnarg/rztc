#![allow(non_upper_case_globals)]

use zt_sys::controller::*;

pub struct Controller {
    rztc_controller: *mut RZTC_Controller,
}

impl Controller {
    /// Creates an instance of controller
    pub fn new() -> Self {
        Self {
            rztc_controller: std::ptr::null_mut(),
        }
    }
}
