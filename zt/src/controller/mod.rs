#![allow(non_upper_case_globals)]

mod callback;
mod error;
mod identity;
mod certificate;
mod networkconfig;

use callback::*;
use error::*;
use zt_sys::controller::*;
use crate::dictionary::Dictionary;
use crate::controller::identity::Identity;
use crate::controller::networkconfig::NetworkConfig;
use num_traits::FromPrimitive;
use failure::Fallible;
use std::collections::VecDeque;
use sha2::Digest;
use ed25519_dalek::{Keypair, Signer, KEYPAIR_LENGTH};
use std::net::Ipv4Addr;

#[derive(Debug, Clone)]
pub struct NetworkRequest {
    pub nwid: u64,
    pub packet_id: u64,
    pub identity: Identity,
    pub metadata: Box<Dictionary>,
}

#[derive(Debug, Clone)]
pub struct Network {
    pub name: String,
    pub id: u32,
    pub prefix: u8,
    pub revision: u64,
    pub public: bool,
    pub broadcast: bool,
    pub multicast_recipient_limit: u64,
    pub mtu: u16,
    pub members: Vec<Member>,
}

#[derive(Debug, Clone)]
pub struct Member {
    pub address: u64,
    pub ip: Ipv4Addr,
}

pub struct Controller {
    rztc_controller: *mut RZTC_Controller,
    networks: Vec<Network>,
    id: u64,
    keypair: Option<Keypair>,
    queue: Box<VecDeque<NetworkRequest>>,
}

impl Controller {
    /// Creates an instance of controller
    pub fn new() -> Self {
        Self {
            rztc_controller: std::ptr::null_mut(),
            networks: Vec::new(),
            id: 0,
            keypair: None,
            queue: Box::new(VecDeque::new()),
        }
    }

    /// Gets called when node receives a network config request
    fn on_request(&mut self, nwid: u64, packet_id: u64, identity: Identity, metadata: Dictionary) {
        self.queue.push_back(NetworkRequest {
            nwid: nwid,
            packet_id: packet_id,
            identity: identity,
            metadata: Box::new(metadata),
        });
    }

    pub fn process_request(&self, req: &NetworkRequest) {
        let mut nc = NetworkConfig::new("my-network-lab", req.nwid, &req.identity, 1).unwrap();
        nc.sign(self.id, self).expect("unable to sign network config");
        let buf = nc.serialize().unwrap();
        unsafe {
            RZTC_Controller_sendConfig(
                self.rztc_controller,
                req.nwid,
                req.packet_id,
                req.identity.address,
                buf.as_ptr() as *const _,
                false
            );
        }
    }

    fn set_keypair(&mut self, id: u64, keypair: Keypair) {
        self.id = id;
        self.keypair = Some(keypair);
    }

    pub fn add_network(&mut self, network: Network) {
        self.networks.push(network);
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



impl ZeroTierSigner for Controller {
    fn sign(&self, data: &[u8]) -> Fallible<[u8; 96]> {
        // Data is signed by hashing the data using SHA-512
        // and signing the first 32 bytes of it. The final
        // signature is then constructed like this:
        //
        // |--           96           --|
        // |--       64      --|-- 32 --|
        // |-------------------|--------|
        // | Ed25519 signature |  hash  |
        // ------------------------------
        match &self.keypair {
            Some(keypair) => {
                let mut signature = [0u8; 96];
                let digest = &sha2::Sha512::digest(data)[..32];
                let signed = keypair.sign(&digest).to_bytes();
                signature[..KEYPAIR_LENGTH].copy_from_slice(&signed);
                signature[KEYPAIR_LENGTH..].copy_from_slice(digest);
                Ok(signature)
            },
            None => Err(SignatureError::NoKeypair.into()),
        }
    }
}

pub trait NetworkConfigProvider {
    fn get_network_config(&self, identity: u64) -> Fallible<NetworkConfig>;
}

pub trait ZeroTierSigner {
    fn sign(&self, data: &[u8]) -> Fallible<[u8; 96]>;
}
