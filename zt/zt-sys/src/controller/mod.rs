use super::*;

pub type RZTC_Controller = ::std::os::raw::c_void;

pub type RZTC_networkRequestCallback = ::std::option::Option<
    unsafe extern "C" fn(
        arg1: *mut RZTC_Controller,
        arg2: *mut ::std::os::raw::c_void,
        arg3: u64,
        arg4: *const libc::sockaddr_storage,
        arg5: u64,
        arg6: u64,
        arg7: *const ::std::os::raw::c_void,
        arg8: u64,
    ),
>;
extern "C" {
    pub fn RZTC_Controller_new(
        controller: *mut *mut RZTC_Controller,
        node: *mut ZT_Node,
        uptr: *mut ::std::os::raw::c_void,
        callback: RZTC_networkRequestCallback,
    ) -> ZT_ResultCode;
}
extern "C" {
    pub fn RZTC_Controller_delete(controller: *mut RZTC_Controller);
}
extern "C" {
    pub fn RZTC_Controller_sendConfig(
        controller: *mut RZTC_Controller,
        nwid: u64,
        requestPacketId: u64,
        dest: u64,
        nc: *const ::std::os::raw::c_void,
        legacy: bool,
    );
}
