#![allow(non_upper_case_globals)]
mod error;
mod callback;

use error::*;
use callback::*;
use zt_sys::*;

use std::ffi::c_void;
use std::os::raw::{c_int, c_uint};
use std::time::{SystemTime, UNIX_EPOCH};
use std::option::Option::Some;

pub struct Node {
    zt_node: *mut ZT_Node,
    config_provider: Box<dyn ConfigurationProvider>,
}

impl Node {
    // Create a new node
    pub fn new(conf_provider: Box<dyn ConfigurationProvider>) -> Result<Node, InternalError> {
        let mut n = Self {
            zt_node: std::ptr::null_mut(),
            config_provider: conf_provider,
        };
        let node_mut_ptr: *mut *mut ZT_Node = &mut n.zt_node;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("unable to get time in millis");
        let now = now.as_millis().try_into().unwrap();
        let cbs = ZT_Node_Callbacks {
            version: 0,
            statePutFunction: Some(state_put_function),
            stateGetFunction: Some(state_get_function),
            wirePacketSendFunction: Some(wire_packet_send_function),
            virtualNetworkConfigFunction: Some(virtual_network_config_function),
            virtualNetworkFrameFunction: Some(virtual_network_frame_function),
            eventCallback: Some(event_callback),
            pathCheckFunction: None,
            pathLookupFunction: None,
        };

        unsafe {
            let c_node: *const Node = &n;
            let _ret = ZT_Node_new(
                node_mut_ptr,
                c_node as *mut c_void,
                0 as *mut c_void,
                &cbs,
                now
            );
        }
        Ok(n)
    }

    fn on_event(&self, event: Event) {
        println!("Got event: {:?}", event);
    }

    fn on_wire_packet(&self, buf: &[u8], socket: i64) -> bool {
        println!("will process packet: {}", socket);
        println!("{}", hex::encode(buf));
        false
    }

    fn set_state(&self, object_type: StateObject, _buf: *const c_void, _len: c_int) {
        match object_type {
            StateObject::PublicIdentity => println!("setting public identity!"),
            StateObject::SecretIdentity => println!("setting secret identity!"),
        }
    }

    fn get_state(&self, object_type: StateObject, buf: *mut c_void, max_len: c_uint) -> c_int {
        let value = match object_type {
            StateObject::PublicIdentity => self.config_provider.get_public_identity(),
            StateObject::SecretIdentity => self.config_provider.get_secret_identity(),
        };

        let s_len = value.len();
        unsafe {
            let buf: *mut [u8] = std::ptr::slice_from_raw_parts_mut(buf as *mut u8, max_len as usize).try_into().unwrap();
            (*buf)[..s_len].copy_from_slice(value.as_bytes());
        }
        return s_len as c_int;
    }

    pub fn process_background_tasks(&self) -> i64 {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("unable to get time in millis");
        let now: i64 = now.as_millis().try_into().unwrap();
        let mut next: i64 = 0;
        unsafe {
            ZT_Node_processBackgroundTasks(
                self.zt_node as *mut ZT_Node,
                0 as *mut c_void,
                now,
                &mut next,
            );
        }
        next
    }

    // TODO: implement
    // pub fn register_controller() {
    //
    // }
}

impl Drop for Node {
    fn drop(&mut self) {
        unsafe {
            ZT_Node_delete(self.zt_node as *mut ZT_Node);
        }
    }
}

pub trait ConfigurationProvider {
    fn get_public_identity(&self) -> String;
    fn get_secret_identity(&self) -> String;
    fn set_public_identity(&self, public_identity: String) -> bool;
    fn set_secret_identity(&self, secret_identity: String) -> bool;
}
