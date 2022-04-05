use super::*;
use zt_sys::*;
use std::ffi::c_void;
use std::os::raw::{c_int, c_uint};
use num_traits::FromPrimitive;

#[derive(Debug, FromPrimitive)]
pub enum Event {
    Up = ZT_Event_ZT_EVENT_UP as isize,
    Down = ZT_Event_ZT_EVENT_DOWN as isize,
    Online = ZT_Event_ZT_EVENT_ONLINE as isize,
    Offline = ZT_Event_ZT_EVENT_OFFLINE as isize,
}

#[derive(Debug, FromPrimitive)]
pub enum StateObject {
    PublicIdentity = ZT_StateObjectType_ZT_STATE_OBJECT_IDENTITY_PUBLIC as isize,
    SecretIdentity = ZT_StateObjectType_ZT_STATE_OBJECT_IDENTITY_SECRET as isize,
}

fn cast_to_node(node: *mut c_void) -> &'static Node {
    unsafe { &*(node as *const Node) }
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
    let n: &Node = cast_to_node(node);
    if let Some(state_object) = StateObject::from_u32(object_type) {
        n.set_state(state_object, data, len);
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
    let n: &Node = cast_to_node(node);
    if let Some(state_object) = StateObject::from_u32(object_type) {
        return n.get_state(state_object, data, len);
    }
    -1
}

// TODO: implement
#[no_mangle]
pub extern "C" fn wire_packet_send_function(
    _n: *mut ZT_Node,
    node: *mut c_void,
    _tptr: *mut c_void,
    socket: i64,
    _address: *const sockaddr_storage,
    data: *const c_void,
    len: c_uint,
    _ttl: c_uint
) -> c_int {
    let n: &Node = cast_to_node(node);
    // unsafe call! We have to trust that ZT_Node reports correct length
    let buf = unsafe{ std::slice::from_raw_parts(data as *const u8, len as usize) };
    !n.on_wire_packet(buf, socket) as c_int
}

#[no_mangle]
pub extern "C" fn event_callback(
    _n: *mut ZT_Node,
    node: *mut c_void,
    _tptr: *mut c_void,
    event_type: ZT_Event,
    _payload: *const c_void
) {
    let n: &Node = cast_to_node(node);
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
