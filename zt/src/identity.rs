extern crate zt_sys;
extern crate libc;

use std::ffi::CString;
use zt_sys::*;

pub struct Identity {
    zt_identity: *const ZT_Identity,
    _private_key: String,
    pub _public_key: String,
    pub _identity: String,
}

impl Identity {
    pub fn new() -> Identity {
        let i = Self {
            zt_identity: std::ptr::null(),
            // TODO: fetch from identity
            _identity: String::from(""),
            _public_key: String::from(""),
            _private_key: String::from(""),
        };
        unsafe {
            let mut zt_identity_ptr: *mut ZT_Identity = i.zt_identity as *mut ZT_Identity;
            let _ret = ZT_Identity_new(&mut zt_identity_ptr);
            let _ret2 = ZT_Identity_generate(i.zt_identity as *mut ZT_Identity);
        }
        return i;
    }

    pub fn from_string(identity: String) -> Identity {
        let i = Self {
            zt_identity: std::ptr::null(),
            // TODO: fetch from identity
            _identity: String::from(""),
            _public_key: String::from(""),
            _private_key: String::from(""),
        };
        let c_id = CString::new(identity).expect("failed to convert identity to CString");
        unsafe {
            let mut zt_identity_ptr: *mut ZT_Identity = i.zt_identity as *mut ZT_Identity;
            let _ret = ZT_Identity_new(&mut zt_identity_ptr);
            let _ret2 = ZT_Identity_fromString(i.zt_identity as *mut ZT_Identity, c_id.as_ptr());
        }
        return i;
    }
}

impl Drop for Identity {
    fn drop(&mut self) {
        unsafe {
            ZT_Identity_delete(self.zt_identity as *mut ZT_Identity);
        }
    }
}
