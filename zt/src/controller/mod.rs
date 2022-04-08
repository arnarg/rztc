#![allow(non_upper_case_globals)]
mod callback;

use callback::*;
use zt_sys::controller::*;
use crate::core::*;
use crate::dictionary::Dictionary;
use num_traits::FromPrimitive;
use failure::Fallible;

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

    /// Gets called when node receives a network config request
    pub fn on_request(&self, nwid: u64, packet_id: u64, identity: u64, _dict: Dictionary) {
        println!("Got network request!");
        println!("  nwid: {:x}", nwid);
        println!("  packet: {:x}", packet_id);
        println!("  identity: {:x}", identity);
    }
}

impl crate::core::Controller for Controller {
    fn init_controller(&self) -> Fallible<*const ()> {
        let cbs = RZTC_Controller_Callbacks {
            networkRequestCallback: Some(on_network_request),
        };
        let controller_ptr: *const *mut RZTC_Controller = &self.rztc_controller;
        let controller: *const Controller = self;

        let ret: RZTC_ResultCode = unsafe {
            RZTC_Controller_new(
                controller_ptr as *mut *mut RZTC_Controller,
                controller as *mut _,
                &cbs,
            )
        };
        match ret {
            0 => Ok(self.rztc_controller as *const _),
            _ => match FatalError::from_u32(ret) {
                Some(err) => Err(err.into()),
                None => Err(FatalError::Internal.into()),
            },
        }
    }
}
