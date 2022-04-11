use super::*;
use zt_sys::*;
use std::ffi::c_void;
use std::os::raw::{c_int, c_uint};
use num_traits::FromPrimitive;

#[derive(Debug, FromPrimitive, PartialEq, Eq)]
pub enum Event {
    Up = ZT_Event_ZT_EVENT_UP as isize,
    Down = ZT_Event_ZT_EVENT_DOWN as isize,
    Online = ZT_Event_ZT_EVENT_ONLINE as isize,
    Offline = ZT_Event_ZT_EVENT_OFFLINE as isize,
}

#[derive(Debug, FromPrimitive, PartialEq, Eq)]
pub enum StateObject {
    PublicIdentity = ZT_StateObjectType_ZT_STATE_OBJECT_IDENTITY_PUBLIC as isize,
    SecretIdentity = ZT_StateObjectType_ZT_STATE_OBJECT_IDENTITY_SECRET as isize,
}

macro_rules! to_node {
    ( $a:expr ) => {
        unsafe { &*($a as *const Node) }
    };
}

macro_rules! to_mut_node {
    ( $a:expr ) => {
        unsafe { &mut *($a as *mut Node) }
    };
}

// Callback functions expected by ZT_Node
#[no_mangle]
pub extern "C" fn state_put_function(
    _n: *mut ZT_Node,
    node: *mut c_void,
    _tptr: *mut c_void,
    object_type: ZT_StateObjectType,
    _id: *const u64,
    data: *const c_void,
    len: c_int
) {
    // Recover the rust native Node through the user pointer
    let n: &Node = to_node!(node);
    // Casting the ZT_StateObjectType from C to rust native enum
    // If it is not recognized we do nothing
    if let Some(state_object) = StateObject::from_u32(object_type) {
        if len < 0 {
            // call delete_state
        } else {
            // unsafe call! We have to trust that ZT_Node reports correct length
            let buf = unsafe{ std::slice::from_raw_parts(data as *const u8, len as usize) };
            n.set_state(state_object, buf);
        }
    }
}

#[no_mangle]
pub extern "C" fn state_get_function(
    _n: *mut ZT_Node,
    node: *mut c_void,
    _tptr: *mut c_void,
    object_type: ZT_StateObjectType,
    _id: *const u64,
    data: *mut c_void,
    len: c_uint
) -> c_int {
    // Recover the rust native Node through the user pointer
    let n: &Node = to_node!(node);
    // Casting the ZT_StateObjectType from C to rust native enum
    if let Some(state_object) = StateObject::from_u32(object_type) {
        // unsafe call! We have to trust that ZT_Node reports correct length
        let buf = unsafe{ std::slice::from_raw_parts_mut(data as *mut u8, len as usize) };
        return n.get_state(state_object, buf) as i32;
    }
    -1
}

#[no_mangle]
pub extern "C" fn wire_packet_send_function(
    _n: *mut ZT_Node,
    node: *mut c_void,
    tptr: *mut c_void,
    socket: i64,
    address: *const sockaddr_storage,
    data: *const c_void,
    len: c_uint,
    _ttl: c_uint
) -> c_int {
    // Recover the rust native Node through the user pointer
    let n: &mut Node = to_mut_node!(node);
    // converting C native sockaddr_storage to rust native SocketAddr
    let addr = unsafe{
        sockaddr_to_addr(&*address, std::mem::size_of::<sockaddr_storage>()).unwrap()
    };
    // unsafe call! We have to trust that ZT_Node reports correct length
    let buf = unsafe{ std::slice::from_raw_parts(data as *const u8, len as usize) };
    // Usually a thread pointer will be passed from C which should contain our PhyWrapper
    // (see Node.process_wire_packet()). But when the node has a controller, any responses
    // coming from the controller will not pass any thread pointer.
    // In those cases we enqueue the packet for processing later. This is more expensive
    // as the C code owns the buffer memory so if we want to store it for later we have to
    // copy the entire buffer data. This might not be terribly expensive to do once in a
    // while (controller responses) but we certainly don't want to do this for every packet.
    if !tptr.is_null() {
        // Recover the rust native PhyWrapper through the thread pointer
        let p: &PhyWrapper = unsafe {
            &*(tptr as *const PhyWrapper)
        };
        !n.on_wire_packet(p.0, buf, socket, addr.clone()) as c_int
    } else {
        n.enqueue_wire_packet(buf, socket, addr.clone());
        0
    }
}

#[no_mangle]
pub extern "C" fn event_callback(
    _n: *mut ZT_Node,
    node: *mut c_void,
    _tptr: *mut c_void,
    event_type: ZT_Event,
    _payload: *const c_void
) {
    // Recover the rust native Node through the user pointer
    let n: &Node = to_node!(node);
    if let Some(ev) = Event::from_u32(event_type) {
        n.on_event(ev);
    }
}

// TODO: implement
#[no_mangle]
pub extern "C" fn virtual_network_config_function(
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
pub extern "C" fn virtual_network_frame_function(
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
pub extern "C" fn path_check_function(
    _n: *mut ZT_Node,
    _node: *mut c_void,
    _tptr: *mut c_void,
    _ztaddress: u64,
    _socket: c_int,
    _address: *const sockaddr_storage
) -> c_int {0}

// TODO: implement
#[no_mangle]
pub extern "C" fn path_lookup_function(
    _n: *mut ZT_Node,
    _node: *mut c_void,
    _tptr: *mut c_void,
    _ztaddress: u64,
    _family: c_int,
    _address: *const sockaddr_storage
) -> c_int {0}
