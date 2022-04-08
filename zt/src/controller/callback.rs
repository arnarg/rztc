use super::Controller;
use zt_sys::controller::*;
use crate::dictionary::Dictionary;

macro_rules! to_controller {
    ( $a:expr ) => {
        unsafe { &*($a as *const Controller) }
    };
}

#[no_mangle]
pub extern "C" fn on_network_request(
        _rztc_controller: *mut RZTC_Controller,
        controller: *mut ::std::os::raw::c_void,
        nwid: u64,
        _sockaddr: *const libc::sockaddr_storage,
        packet_id: u64,
        identity: u64,
        metadata_dict: *const ::std::os::raw::c_void,
        max_len: u64,
) {
    // Recover the rust native Controller through the user pointer
    let c: &Controller = to_controller!(controller);
    // Cast metadata_dict to slice
    let buf = unsafe{ std::slice::from_raw_parts(metadata_dict as *const u8, max_len as usize) };
    let index: usize = match buf.iter().position(|x| *x == 0) {
        Some(i) => i,
        None => max_len as usize,
    };
    let dict = Dictionary::from(&buf[..index]);
    c.on_request(nwid, packet_id, identity, dict);
}
