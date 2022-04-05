#![allow(non_upper_case_globals)]
mod error;
mod callback;

pub use error::*;
use callback::*;
use zt_sys::*;

use std::ffi::c_void;
use std::time::{SystemTime, UNIX_EPOCH};
use std::option::Option::Some;
use num_traits::FromPrimitive;

pub struct Node {
    zt_node: *mut ZT_Node,
    config_provider: Box<dyn ConfigurationProvider>,
}

impl Node {
    /// Creates an instance of node
    pub fn new(conf_provider: Box<dyn ConfigurationProvider>) -> Result<Node, FatalError> {
        let mut n = Self {
            zt_node: std::ptr::null_mut(),
            config_provider: conf_provider,
        };

        // Get current time in milliseconds since epoch.
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("unable to get time in millis");
        let now = now.as_millis().try_into().unwrap();

        // Construct callback struct ZT_Node_new is expecting.
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

        // Create a double pointer so ZT_Node_new() can set the address of the instance of
        // ZT_Node. This pointer is used in any subsequent call to ZT_*.
        let node_mut_ptr: *mut *mut ZT_Node = &mut n.zt_node;
        // Create a user pointer to the node to be used in callbacks.
        let node: *const Node = &n;

        let ret: ZT_ResultCode = unsafe { ZT_Node_new(node_mut_ptr, node as *mut c_void, 0 as *mut c_void, &cbs, now) };
        match ret {
            0 => Ok(n),
            _ => match FatalError::from_u32(ret) {
                Some(err) => Err(err),
                None => Err(FatalError::Internal),
            },
        }
    }

    /// Perform periodic background operations
    ///
    /// Returns next deadline when it should run in milliseconds since epoch
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

    // Gets called from C (through a callback wrapper) when an event occurs
    fn on_event(&self, event: Event) {
        println!("Got event: {:?}", event);
    }

    // Gets called from C (through a callback wrapper) when a packet should be
    // sent to a socket
    fn on_wire_packet(&self, buf: &[u8], socket: i64) -> bool {
        println!("will process packet: {}", socket);
        println!("{}", hex::encode(buf));
        false
    }

    // Gets called from C (through a callback wrapper) when the node wants to
    // save state
    fn set_state(&self, object_type: StateObject, buf: &[u8]) {
        match object_type {
            StateObject::PublicIdentity => self.config_provider.set_public_identity(buf),
            StateObject::SecretIdentity => self.config_provider.set_secret_identity(buf),
        };
    }

    // Gets called from C (through a callback wrapper) when the node wants to
    // get state
    fn get_state(&self, object_type: StateObject, buf: &mut [u8]) -> usize {
        let value = match object_type {
            StateObject::PublicIdentity => self.config_provider.get_public_identity(),
            StateObject::SecretIdentity => self.config_provider.get_secret_identity(),
        };

        let len = value.len();
        buf[..len].copy_from_slice(value.as_bytes());
        len
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
    fn set_public_identity(&self, public_identity: &[u8]) -> bool;
    fn set_secret_identity(&self, secret_identity: &[u8]) -> bool;
}
