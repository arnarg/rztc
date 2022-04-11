#![allow(non_upper_case_globals)]

mod callback;
mod error;

use callback::*;
use zt_sys::controller::*;
use crate::dictionary::Dictionary;
use num_traits::FromPrimitive;
use failure::Fallible;
use queues::{Queue, IsQueue};

#[derive(Debug, Clone)]
pub struct NetworkRequest {
    pub nwid: u64,
    pub packet_id: u64,
    pub identity: u64,
    pub metadata: Box<Dictionary>,
}

pub struct Controller {
    rztc_controller: *mut RZTC_Controller,
    queue: Queue<NetworkRequest>,
}

impl Controller {
    /// Creates an instance of controller
    pub fn new() -> Self {
        Self {
            rztc_controller: std::ptr::null_mut(),
            queue: queue![],
        }
    }

    /// Gets called when node receives a network config request
    pub fn on_request(&mut self, nwid: u64, packet_id: u64, identity: u64, metadata: Dictionary) {
        self.queue.add(NetworkRequest {
            nwid: nwid,
            packet_id: packet_id,
            identity: identity,
            metadata: Box::new(metadata),
        });
    }

    pub fn process_request(&self, req: &NetworkRequest) {
        // Crashes!
        unsafe {
            RZTC_Controller_sendError(
                self.rztc_controller,
                req.nwid,
                req.packet_id,
                req.identity,
                error::NetworkError::NotFound as u32,
                0 as *const std::ffi::c_void,
                0
            );
        }
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
            _ => match error::FatalError::from_u32(ret) {
                Some(err) => Err(err.into()),
                None => Err(error::FatalError::Internal.into()),
            },
        }
    }

    fn process_background_tasks(&mut self) -> Fallible<()> {
        while self.queue.size() > 0 {
            match self.queue.peek() {
                Ok(req) => {
                    println!("Got network config request from '{:x}' for network '{:x}'", req.identity, req.nwid);
                    self.process_request(&req);
                },
                Err(error) => println!("unable to get request from queue: {}", error),
            };
            self.queue.remove();
        }
        Ok(())
    }
}
