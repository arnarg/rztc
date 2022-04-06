#![allow(non_upper_case_globals)]
mod error;
mod callback;

pub use error::*;
pub use callback::StateObject;
use callback::*;
use zt_sys::*;

use std::time::{SystemTime, UNIX_EPOCH};
use std::option::Option::Some;
use std::cell::Cell;
use std::net::SocketAddr;
use pnet_sys::{addr_to_sockaddr, sockaddr_to_addr};
use libc::sockaddr_storage;
use num_traits::FromPrimitive;
use failure::Fallible;

pub struct Node {
    zt_node: *mut ZT_Node,
    online: Cell<bool>,
    config_provider: Box<dyn ConfigurationProvider>,
}

impl Node {
    /// Creates an instance of node
    pub fn new(conf_provider: Box<dyn ConfigurationProvider>) -> Fallible<Node> {
        Ok(Self {
            zt_node: std::ptr::null_mut(),
            online: Cell::new(false),
            config_provider: conf_provider,
        })
    }

    // If we create a new instance of Node and pass a pointer to that as the user
    // pointer to ZT_Node_new before returning the Node to the caller of new()
    // the pointer value is different from what the caller gets.
    //
    // As a result whoever receives the user pointer doesn't have the "same"
    // Node as the caller of new().
    //
    // on_event() needs to mutate self.online and this wasn't working.
    //
    // As a workaround I return the Node immediately and initialize it later
    // once we have the correct pointer value in this init private function
    // and just call it in every public function if self.zt_node is still
    // a null pointer.
    fn init(&self, now: i64) -> Fallible<()> {
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
        let node_mut_ptr: *const *mut ZT_Node = &self.zt_node;
        // Create a user pointer to the node to be used in callbacks.
        let node: *const Node = self;

        let ret: ZT_ResultCode = unsafe {
            ZT_Node_new(
                node_mut_ptr as *mut *mut ZT_Node,
                node as *mut _, // Reference to self as user pointer
                0 as *mut _, // Thread pointer not needed here
                &cbs,
                now
            )
        };
        match ret {
            0 => Ok(()),
            _ => match FatalError::from_u32(ret) {
                Some(err) => Err(err.into()),
                None => Err(FatalError::Internal.into()),
            },
        }
    }

    /// Perform periodic background operations
    ///
    /// Returns next deadline when it should run in milliseconds since epoch
    pub fn process_background_tasks(&self, phy: &dyn PhyProvider) -> Fallible<i64> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("unable to get time in millis");
        let now: i64 = now.as_millis().try_into().unwrap();
        let mut next: i64 = 0;

        // Do we still need to initialize the node?
        // See comment above init().
        if self.zt_node.is_null() {
            self.init(now)?;
        }

        // Calling processBackgroundTasks in C calls a long chain of functions until
        // eventually calling back into rust in wire_packet_send_function callback.
        //
        // Because I don't want Node to own `dyn PhyProvider` I wrap a pointer to
        // the PhyProvider in a struct and convert that to a raw pointer that I can
        // pass to C as a thread pointer. This thread pointer is passed down the
        // entire chain of functions and in wire_packet_send_function we can cast
        // it back into our wrapper and use the PhyProvider.
        //
        // This is of course very unsafe.
        let phy_wrapper = Box::new(PhyWrapper(phy));
        let phy_wrapper_ptr: *mut PhyWrapper = Box::into_raw(phy_wrapper);

        // Call into C
        let ret = unsafe {
            ZT_Node_processBackgroundTasks(
                self.zt_node as *mut ZT_Node,
                phy_wrapper_ptr as *mut _,
                now,
                &mut next
            )
        };

        // Reclaim the PhyProvider wrapper.
        //
        // In wire_packet_send_function I'm only dereferncing the wrapper pointer
        // not taking ownership of the data.
        unsafe { Box::from_raw(phy_wrapper_ptr) };

        match ret {
            0 => Ok(next),
            _ => match FatalError::from_u32(ret) {
                Some(err) => Err(err.into()),
                None => Err(FatalError::Internal.into()),
            },
        }
    }

    pub fn process_wire_packet(&self, phy: &dyn PhyProvider, buf: &[u8], len: usize, addr: &SocketAddr, socket: i64) -> Fallible<i64> {
        // Get current time in millis since epoch
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("unable to get time in millis");
        let now: i64 = now.as_millis().try_into().unwrap();

        // Do we still need to initialize the node?
        // See comment above init().
        if self.zt_node.is_null() {
            self.init(now)?;
        }

        // Variable for next deadline
        let mut next: i64 = 0;

        // Calling processWirePacket in C calls a long chain of functions until
        // eventually (sometimes) calling back into rust in wire_packet_send_function
        // callback.
        //
        // Because I don't want Node to own `dyn PhyProvider` I wrap a pointer to
        // the PhyProvider in a struct and convert that to a raw pointer that I can
        // pass to C as a thread pointer. This thread pointer is passed down the
        // entire chain of functions and in wire_packet_send_function we can cast
        // it back into our wrapper and use the PhyProvider.
        //
        // This is of course very unsafe.
        let phy_wrapper = Box::new(PhyWrapper(phy));
        let phy_wrapper_ptr: *mut PhyWrapper = Box::into_raw(phy_wrapper);

        let ret = unsafe {
            use std::alloc::{alloc, dealloc, Layout};

            let layout = Layout::new::<sockaddr_storage>();
            let ptr = alloc(layout);
            let sockaddr = &mut *(ptr as *mut sockaddr_storage);

            addr_to_sockaddr(*addr, sockaddr);

            let res = ZT_Node_processWirePacket(
                self.zt_node as *mut ZT_Node,
                phy_wrapper_ptr as *mut _,
                now,
                socket,
                sockaddr,
                buf.as_ptr() as *const _,
                len as u32,
                &mut next
            );

            dealloc(ptr, layout);
            res
        };

        // Reclaim the PhyProvider wrapper.
        //
        // In wire_packet_send_function I'm only dereferncing the wrapper pointer
        // not taking ownership of the data.
        unsafe { Box::from_raw(phy_wrapper_ptr) };

        match ret {
            0 => Ok(next),
            _ => match FatalError::from_u32(ret) {
                Some(err) => Err(err.into()),
                None => Err(FatalError::Internal.into()),
            },
        }
    }

    /// Returns online status of node.
    pub fn is_online(&self) -> bool { self.online.get() }

    // Gets called from C (through a callback wrapper) when an event occurs
    fn on_event(&self, event: Event) {
        match event {
            Event::Online => self.online.set(true),
            Event::Offline => self.online.set(false),
            _ => (),
        }
        println!("Got event: {:?}", event);
    }

    // Gets called from C (through a callback wrapper) when a packet should be
    // sent to a socket
    fn on_wire_packet(&self, phy: &dyn PhyProvider, buf: &[u8], socket: i64, addr: SocketAddr) -> bool {
        if addr.is_ipv4() {
            match socket {
                -1 => phy.send_all(&addr, buf),
                _ => phy.send(&addr, socket, buf),
            };
        }
        false
    }

    // Gets called from C (through a callback wrapper) when the node wants to
    // save state
    fn set_state(&self, object_type: StateObject, buf: &[u8]) {
        let _ret = self.config_provider.set_state(object_type, buf);
    }

    // Gets called from C (through a callback wrapper) when the node wants to
    // get state
    fn get_state(&self, object_type: StateObject, buf: &mut [u8]) -> i32 {
        if let Ok(value) = self.config_provider.get_state(object_type) {
            let len = value.len();
            buf[..len].copy_from_slice(&value);
            return len as i32;
        }
        -1
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
    fn get_state(&self, object_type: StateObject) -> Fallible<Vec<u8>>;
    fn set_state(&self, object_type: StateObject, data: &[u8]) -> Fallible<()>;
}

struct PhyWrapper<'a>(&'a dyn PhyProvider);

pub trait PhyProvider {
    fn send(&self, address: &SocketAddr, socket: i64, buf: &[u8]) -> usize;
    fn send_all(&self, address: &SocketAddr, buf: &[u8]) -> usize;
}
