#![allow(non_upper_case_globals)]

extern crate zt_sys;
extern crate libc;

use std::ffi::CString;
use crate::core::ZTError;
use zt_sys::*;

pub struct Identity {
    zt_identity: *const ZT_Identity,
    _private_key: String,
    pub _public_key: String,
    pub _identity: String,
}

impl Identity {
    pub fn new() -> Result<Identity, ZTError> {
        let i = Self {
            zt_identity: Self::new_zt_identity()?,
            // TODO: fetch from identity
            _identity: String::from(""),
            _public_key: String::from(""),
            _private_key: String::from(""),
        };
        let ret: ZT_ResultCode;
        unsafe {
            ret = ZT_Identity_generate(i.zt_identity as *mut ZT_Identity);
        }
        if ret == ZT_ResultCode_ZT_RESULT_FATAL_ERROR_OUT_OF_MEMORY {
            return Err(ZTError::OutOfMemory)
        }
        Ok(i)
    }

    pub fn from_string(identity: String) -> Result<Identity, ZTError> {
        let i = Self {
            zt_identity: Self::new_zt_identity()?,
            // TODO: fetch from identity
            _identity: String::from(""),
            _public_key: String::from(""),
            _private_key: String::from(""),
        };
        let c_id = CString::new(identity).expect("failed to convert identity to CString");
        let success: bool;
        unsafe {
            success = ZT_Identity_fromString(i.zt_identity as *mut ZT_Identity, c_id.as_ptr());
        }
        if !success {
            return Err(ZTError::BadParameter)
        }
        Ok(i)
    }

    fn new_zt_identity() -> Result<*const ZT_Identity, ZTError> {
        let mut zt_identity_ptr: *mut ZT_Identity = std::ptr::null_mut();
        let ret: ZT_ResultCode;
        unsafe {
            ret = ZT_Identity_new(&mut zt_identity_ptr);
        }
        match ret {
            ZT_ResultCode_ZT_RESULT_FATAL_ERROR_OUT_OF_MEMORY=>Err(ZTError::OutOfMemory),
            ZT_ResultCode_ZT_RESULT_FATAL_ERROR_DATA_STORE_FAILED=>Err(ZTError::DataStoreFailed),
            ZT_ResultCode_ZT_RESULT_FATAL_ERROR_INTERNAL=>Err(ZTError::Internal),
            _=>Ok(zt_identity_ptr as *const ZT_Identity),
        }
    }
}

impl Drop for Identity {
    fn drop(&mut self) {
        unsafe {
            ZT_Identity_delete(self.zt_identity as *mut ZT_Identity);
        }
    }
}
