#![allow(non_upper_case_globals)]

use zt_sys::ZT_ResultCode;
use zt_sys::controller::*;
use crate::core::*;
use crate::dictionary::Dictionary;
use num_traits::FromPrimitive;
use failure::Fallible;

// #[derive(Debug, Fail, FromPrimitive)]
// pub enum FatalError {
//     #[fail(display = "out of memory")]
//     OutOfMemory = RZTC_ResultCode_RZTC_RESULT_FATAL_ERROR_OUT_OF_MEMORY as isize,
//     #[fail(display = "data store failed")]
//     DataStoreFailed = RZTC_ResultCode_RZTC_RESULT_FATAL_ERROR_DATA_STORE_FAILED as isize,
//     #[fail(display = "internal error")]
//     Internal = RZTC_ResultCode_RZTC_RESULT_FATAL_ERROR_INTERNAL as isize,
// }

macro_rules! to_controller {
    ( $a:expr ) => {
        unsafe { &*($a as *const Controller) }
    };
}

// TODO: Implement
#[no_mangle]
pub extern "C" fn on_network_request(
        _rztc_controller: *mut RZTC_Controller,
        controller: *mut ::std::os::raw::c_void,
        _nwid: u64,
        _sockaddr: *const libc::sockaddr_storage,
        _packet_id: u64,
        _identity: u64,
        metadata_dict: *const ::std::os::raw::c_void,
        max_len: u64,
) {
    // Recover the rust native Controller through the user pointer
    let c: &Controller = to_controller!(controller);
    // Cast metadata_dict to slice
    let buf = unsafe{ std::slice::from_raw_parts(metadata_dict as *const u8, max_len as usize) };
    let dict = Dictionary::from(buf);
    println!("{:?}", dict);
    println!("{}", dict.get_i64("vend").unwrap());
    println!("{}", dict.get_u64("mcr").unwrap());
    c.hello();
}

pub struct Controller {
    rztc_controller: *mut RZTC_Controller,
}

impl Controller {
    /// Creates an instance of controller
    pub fn new() -> Self {
        Self {
            rztc_controller: std::ptr::null_mut(),
        }
    }

    pub fn hello(&self) {
        println!("Hello!");
    }
}

impl crate::core::Controller for Controller {
    fn init_controller(&self) -> Fallible<*const ()> {
        let cbs = RZTC_Controller_Callbacks {
            networkRequestCallback: Some(on_network_request),
        };
        let controller_ptr: *const *mut RZTC_Controller = &self.rztc_controller;
        let controller: *const Controller = self;

        let ret: ZT_ResultCode = unsafe {
            RZTC_Controller_new(
                controller_ptr as *mut *mut RZTC_Controller,
                controller as *mut _,
                &cbs,
            )
        };
        match ret {
            0 => Ok(self.rztc_controller as *const _),
            _ => match FatalError::from_u32(ret) {
                Some(err) => Err(err.into()),
                None => Err(FatalError::Internal.into()),
            },
        }
    }
}
