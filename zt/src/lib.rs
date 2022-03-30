extern crate zt_sys;
extern crate libc;

pub mod identity;

pub mod core {
    use std::ffi::c_void;
    use std::os::raw::{c_int, c_uint};
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::option::Option;
    use zt_sys::*;

    pub struct Node {
        node: *const ZT_Node,
        identity: crate::identity::Identity,
    }

    impl Node {
        pub fn new() -> Node {
            let n = Self {
                node: std::ptr::null(),
                identity: crate::identity::Identity {},
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
                let mut zt_node: *mut ZT_Node = n.node as *mut ZT_Node;
                let c_node: *const Node = &n;
                let ret = ZT_Node_new(
                    &mut zt_node,
                    c_node as *mut c_void,
                    0 as *mut c_void,
                    &cbs,
                    now_millis
                );
            }
            return n
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
                ZT_Node_delete(self.node as *mut ZT_Node);
            }
        }
    }

    // Callback functions expected by ZT_Node
    // TODO: implement
    #[no_mangle]
    extern "C" fn state_put_function(
        n: *mut ZT_Node,
        node: *mut c_void,
        tptr: *mut c_void,
        object_type: ZT_StateObjectType,
        id: *const u64,
        data: *const c_void,
        len: c_int
    ) {}

    // TODO: implement
    #[no_mangle]
    extern "C" fn state_get_function(
        n: *mut ZT_Node,
        node: *mut c_void,
        tptr: *mut c_void,
        object_type: ZT_StateObjectType,
        id: *const u64,
        data: *mut c_void,
        len: c_uint
    ) -> c_int {0}

    // TODO: implement
    #[no_mangle]
    extern "C" fn wire_packet_send_function(
        n: *mut ZT_Node,
        node: *mut c_void,
        tptr: *mut c_void,
        socket: i64,
        address: *const sockaddr_storage,
        data: *const c_void,
        len: c_uint,
        ttl: c_uint
    ) -> c_int {0}

    // TODO: implement
    #[no_mangle]
    extern "C" fn event_callback(
        n: *mut ZT_Node,
        node: *mut c_void,
        tptr: *mut c_void,
        event_type: ZT_Event,
        payload: *const c_void
    ) {}

    // TODO: implement
    #[no_mangle]
    extern "C" fn virtual_network_config_function(
        n: *mut ZT_Node,
        node: *mut c_void,
        tptr: *mut c_void,
        nwid: u64,
        user: *mut *mut c_void,
        op: ZT_VirtualNetworkConfigOperation,
        config: *const ZT_VirtualNetworkConfig
    ) -> c_int {0}

    // TODO: implement
    #[no_mangle]
    extern "C" fn virtual_network_frame_function(
        n: *mut ZT_Node,
        node: *mut c_void,
        tptr: *mut c_void,
        nwid: u64,
        user: *mut *mut c_void,
        source: u64,
        destination: u64,
        ether_type: c_uint,
        vlan_id: c_uint,
        data: *const c_void,
        len: c_uint
    ) {}

    // TODO: implement
    #[no_mangle]
    extern "C" fn path_check_function(
        n: *mut ZT_Node,
        node: *mut c_void,
        tptr: *mut c_void,
        ztaddress: u64,
        socket: c_int,
        address: *const sockaddr_storage
    ) -> c_int {0}

    // TODO: implement
    #[no_mangle]
    extern "C" fn path_lookup_function(
        n: *mut ZT_Node,
        node: *mut c_void,
        tptr: *mut c_void,
        ztaddress: u64,
        family: c_int,
        address: *const sockaddr_storage
    ) -> c_int {0}
}
