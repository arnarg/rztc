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
use std::collections::VecDeque;

macro_rules! maybe_init {
    ( $a:expr ) => {
        if $a.zt_node.is_null() {
            $a.init()?;
        }
    }
}

macro_rules! handle_res {
    ( $a:expr, $b:expr ) => {
        match $a {
            0 => Ok($b),
            _ => match FatalError::from_u32($a) {
                Some(err) => Err(err.into()),
                None => Err(FatalError::Internal.into()),
            },
        }
    }
}

macro_rules! log_packet {
    ( $a:expr ) => {
        if $a.len() > 20 {
            let id = &$a[..8];
            let dest = &$a[8..13];
            let src = &$a[13..18];
            let meta = $a[18];
            let flags: u8 = meta >> 6;
            let cipher: u8 = (meta >> 3) & 0b111;
            let cipher = match cipher {
                0 => "none",
                1 => "C25519/POLY1305/SALSA2012",
                3 => "AES-GMAC-SIV",
                _ => "unknown",
            };
            let hops: u8 = meta & 0b111;
            print!(
                "{}: {} -> {} (flags: 0b{:02b}, cipher: {}, hops: {})",
                hex::encode(id),
                hex::encode(src),
                hex::encode(dest),
                flags,
                cipher,
                hops
            );
            if $a.len() > 30 && cipher == "none" && $a[27] == 1 {
                print!(" HELLO");
            }
            print!("\n");
        }
    };
}

#[derive(Clone)]
pub struct WirePacket {
    socket: i64,
    address: SocketAddr,
    buffer: Vec<u8>,
}

pub struct Node {
    zt_node: *mut ZT_Node,
    online: Cell<bool>,
    state_provider: Box<dyn StateProvider>,
    controller: Option<Box<dyn Controller>>,
    packet_queue: Box<VecDeque<WirePacket>>,
}

impl Node {
    /// Creates an instance of node
    pub fn new(conf_provider: Box<dyn StateProvider>) -> Fallible<Node> {
        Ok(Self {
            zt_node: std::ptr::null_mut(),
            online: Cell::new(false),
            state_provider: conf_provider,
            controller: None,
            packet_queue: Box::new(VecDeque::new()),
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
    fn init(&self) -> Fallible<()> {
        // Get current time in millis since epoch
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("unable to get time in millis");
        let now: i64 = now.as_millis().try_into().unwrap();

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
        handle_res!(ret, ())
    }

    /// Perform periodic background operations
    ///
    /// Returns next deadline when it should run in milliseconds since epoch
    pub fn process_background_tasks(&mut self, phy: &dyn PhyProvider) -> Fallible<i64> {
        maybe_init!(self);

        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("unable to get time in millis");
        let now: i64 = now.as_millis().try_into().unwrap();
        let mut next: i64 = 0;


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

        // Process queued packets
        while self.packet_queue.len() > 0 {
            match self.packet_queue.pop_front() {
                Some(packet) => {
                    self.on_wire_packet(phy, &packet.buffer[..], packet.socket, packet.address);
                },
                None => println!("no item in queue"),
            }
        }

        // Run controller background tasks
        if let Some(controller) = &mut self.controller {
            controller.process_background_tasks().unwrap();
        }

        handle_res!(ret, next)
    }

    pub fn process_wire_packet(&self, phy: &dyn PhyProvider, buf: &[u8], len: usize, addr: &SocketAddr, socket: i64) -> Fallible<i64> {
        log_packet!(buf);
        maybe_init!(self);

        // Get current time in millis since epoch
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("unable to get time in millis");
        let now: i64 = now.as_millis().try_into().unwrap();

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

        handle_res!(ret, next)
    }

    pub fn enqueue_wire_packet(&mut self, buf: &[u8], socket: i64, addr: SocketAddr) {
        let packet = WirePacket {
            socket: socket,
            address: addr,
            buffer: Vec::from(buf),
        };
        self.packet_queue.push_back(packet);
    }

    pub fn add_local_interface_address(&self, address: &SocketAddr) -> Fallible<()> {
        unsafe {
            use std::alloc::{alloc, dealloc, Layout};

            let layout = Layout::new::<sockaddr_storage>();
            let ptr = alloc(layout);
            let sockaddr = &mut *(ptr as *mut sockaddr_storage);

            addr_to_sockaddr(*address, sockaddr);

            let res = ZT_Node_addLocalInterfaceAddress(self.zt_node as *mut ZT_Node, sockaddr);

            dealloc(ptr, layout);
            match res != 0 {
                true => Ok(()),
                false => Err(FatalError::Internal.into()),
            }
        }
    }

    pub fn clear_local_interface_addresses(&self) {
        unsafe {
            ZT_Node_clearLocalInterfaceAddresses(self.zt_node as *mut ZT_Node);
        }
    }

    /// Returns online status of node.
    pub fn is_online(&self) -> bool { self.online.get() }

    /// Returns version of libzerotierone
    pub fn version(&self) -> String {
        let mut major: i32 = 0;
        let mut minor: i32 = 0;
        let mut patch: i32 = 0;
        unsafe { ZT_version(&mut major, &mut minor, &mut patch) };
        format!("{}.{}.{}", major, minor, patch)
    }

    // Gets called from C (through a callback wrapper) when an event occurs
    fn on_event(&self, event: Event) {
        match event {
            Event::Online => self.online.set(true),
            Event::Offline => self.online.set(false),
            _ => (),
        }
    }

    // Gets called from C (through a callback wrapper) when a packet should be
    // sent to a socket
    fn on_wire_packet(&self, phy: &dyn PhyProvider, buf: &[u8], socket: i64, addr: SocketAddr) -> bool {
        log_packet!(buf);
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
        let _ret = self.state_provider.set_state(object_type, buf);
    }

    // Gets called from C (through a callback wrapper) when the node wants to
    // get state
    fn get_state(&self, object_type: StateObject, buf: &mut [u8]) -> i32 {
        if let Ok(value) = self.state_provider.get_state(object_type) {
            let len = value.len();
            buf[..len].copy_from_slice(&value);
            return len as i32;
        }
        -1
    }

    pub fn register_controller(&mut self, controller: Box<dyn Controller>) -> Fallible<()> {
        maybe_init!(self);

        let ctrl_ptr = controller.init_controller()?;
        unsafe {
            ZT_Node_setNetconfMaster(
                self.zt_node,
                ctrl_ptr as *mut _,
            )
        };
        self.controller = Some(controller);
        Ok(())
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        unsafe {
            ZT_Node_delete(self.zt_node as *mut ZT_Node);
        }
    }
}

pub trait StateProvider {
    fn get_state(&self, object_type: StateObject) -> Fallible<Vec<u8>>;
    fn set_state(&self, object_type: StateObject, data: &[u8]) -> Fallible<()>;
}

struct PhyWrapper<'a>(&'a dyn PhyProvider);

pub trait PhyProvider {
    fn send(&self, address: &SocketAddr, socket: i64, buf: &[u8]) -> usize;
    fn send_all(&self, address: &SocketAddr, buf: &[u8]) -> usize;
}

pub trait Controller {
    fn init_controller(&self) -> Fallible<*const ()>;
    fn process_background_tasks(&mut self) -> Fallible<()>;
}
