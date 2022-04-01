extern crate zt_sys;
extern crate libc;

pub mod identity;

pub mod core {
    use std::ffi::c_void;
    use std::os::raw::{c_int, c_uint};
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::option::Option::Some;
    use crate::identity::Identity;
    use zt_sys::*;

    #[derive(Debug)]
    pub enum ZTError {
        OutOfMemory,
        DataStoreFailed,
        Internal,
        NotFound,
        UnsupportedOperation,
        BadParameter,
    }

    pub struct Node {
        zt_node: *const ZT_Node,
        _identity: Identity,
    }

    impl Node {
        // Create a new node
        pub fn new() -> Result<Node, ZTError> {
            let n = Self {
                zt_node: std::ptr::null(),
                _identity: Identity::new()?,
            };
            let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("unable to get time in millis");
            let now_millis = now.as_millis().try_into().unwrap();
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
                let mut zt_node_ptr: *mut ZT_Node = n.zt_node as *mut ZT_Node;
                let c_node: *const Node = &n;
                let _ret = ZT_Node_new(
                    &mut zt_node_ptr,
                    c_node as *mut c_void,
                    0 as *mut c_void,
                    &cbs,
                    now_millis
                );
            }
            Ok(n)
        }

        // pub fn process_background_tasks() -> ZT_ResultCode {
        //
        // }
        //
        // pub fn process_wire_packet() -> ZT_ResultCode {
        //
        // }
        //
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

    // Callback functions expected by ZT_Node
    // TODO: implement
    #[no_mangle]
    extern "C" fn state_put_function(
        _n: *mut ZT_Node,
        _node: *mut c_void,
        _tptr: *mut c_void,
        _object_type: ZT_StateObjectType,
        _id: *const u64,
        _data: *const c_void,
        _len: c_int
    ) {}

    // TODO: implement
    #[no_mangle]
    extern "C" fn state_get_function(
        _n: *mut ZT_Node,
        _node: *mut c_void,
        _tptr: *mut c_void,
        _object_type: ZT_StateObjectType,
        _id: *const u64,
        _data: *mut c_void,
        _len: c_uint
    ) -> c_int {0}

    // TODO: implement
    #[no_mangle]
    extern "C" fn wire_packet_send_function(
        _n: *mut ZT_Node,
        _node: *mut c_void,
        _tptr: *mut c_void,
        _socket: i64,
        _address: *const sockaddr_storage,
        _data: *const c_void,
        _len: c_uint,
        _ttl: c_uint
    ) -> c_int {0}

    // TODO: implement
    #[no_mangle]
    extern "C" fn event_callback(
        _n: *mut ZT_Node,
        _node: *mut c_void,
        _tptr: *mut c_void,
        _event_type: ZT_Event,
        _payload: *const c_void
    ) {}

    // TODO: implement
    #[no_mangle]
    extern "C" fn virtual_network_config_function(
        _n: *mut ZT_Node,
        _node: *mut c_void,
        _tptr: *mut c_void,
        _nwid: u64,
        _user: *mut *mut c_void,
        _op: ZT_VirtualNetworkConfigOperation,
        _config: *const ZT_VirtualNetworkConfig
    ) -> c_int {0}

    // TODO: implement
    #[no_mangle]
    extern "C" fn virtual_network_frame_function(
        _n: *mut ZT_Node,
        _node: *mut c_void,
        _tptr: *mut c_void,
        _nwid: u64,
        _user: *mut *mut c_void,
        _source: u64,
        _destination: u64,
        _ether_type: c_uint,
        _vlan_id: c_uint,
        _data: *const c_void,
        _len: c_uint
    ) {}

    // TODO: implement
    #[no_mangle]
    extern "C" fn path_check_function(
        _n: *mut ZT_Node,
        _node: *mut c_void,
        _tptr: *mut c_void,
        _ztaddress: u64,
        _socket: c_int,
        _address: *const sockaddr_storage
    ) -> c_int {0}

    // TODO: implement
    #[no_mangle]
    extern "C" fn path_lookup_function(
        _n: *mut ZT_Node,
        _node: *mut c_void,
        _tptr: *mut c_void,
        _ztaddress: u64,
        _family: c_int,
        _address: *const sockaddr_storage
    ) -> c_int {0}
}
