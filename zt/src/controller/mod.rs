#![allow(non_upper_case_globals)]

mod callback;
mod error;
mod identity;
mod certificate;
mod networkconfig;

use callback::*;
use zt_sys::controller::*;
use crate::dictionary::Dictionary;
use crate::controller::identity::Identity;
use num_traits::FromPrimitive;
use failure::Fallible;
use std::collections::VecDeque;
use ed25519_dalek::Keypair;

const ZT_NETWORKCONFIG_DICT_CAPACITY: usize = 484456;

#[derive(Debug, Clone)]
pub struct NetworkRequest {
    pub nwid: u64,
    pub packet_id: u64,
    pub identity: Identity,
    pub metadata: Box<Dictionary>,
}

pub struct Controller {
    rztc_controller: *mut RZTC_Controller,
    keypair: Option<Keypair>,
    queue: Box<VecDeque<NetworkRequest>>,
}

impl Controller {
    /// Creates an instance of controller
    pub fn new() -> Self {
        Self {
            rztc_controller: std::ptr::null_mut(),
            keypair: None,
            queue: Box::new(VecDeque::new()),
        }
    }

    /// Gets called when node receives a network config request
    pub fn on_request(&mut self, nwid: u64, packet_id: u64, identity: Identity, metadata: Dictionary) {
        self.queue.push_back(NetworkRequest {
            nwid: nwid,
            packet_id: packet_id,
            identity: identity,
            metadata: Box::new(metadata),
        });
    }

    pub fn process_request(&self, req: &NetworkRequest) {
        unsafe {
            RZTC_Controller_sendError(
                self.rztc_controller,
                req.nwid,
                req.packet_id,
                req.identity.address,
                error::NetworkError::AccessDenied as u32,
                std::ptr::null(),
                0
            );
        }
    }

    fn set_keypair(&mut self, keypair: Keypair) {
        self.keypair = Some(keypair);
    }
}

impl crate::core::Controller for Controller {
    fn init_controller(&self) -> Fallible<*const ()> {
        let cbs = RZTC_Controller_Callbacks {
            initCallback: Some(init_controller),
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
        while self.queue.len() > 0 {
            match self.queue.pop_front() {
                Some(req) => {
                    println!("Got network config request from '{:x}' for network '{:x}'", req.identity.address, req.nwid);
                    self.process_request(&req);
                },
                None => println!("no item in queue"),
            };
        }
        Ok(())
    }
}
