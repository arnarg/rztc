#![allow(non_upper_case_globals)]

extern crate zt_sys;
extern crate libc;

use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use crate::core::ZTError;
use zt_sys::*;

pub struct Identity {
    zt_identity: *const ZT_Identity,
    _private_key: String,
    pub _public_key: String,
    pub _identity: String,
}

impl Identity {
    // Generate a new identity
    pub fn new() -> Result<Identity, ZTError> {
        let zt_identity = Self::new_zt_identity()?;
        unsafe {
            let ret = ZT_Identity_generate(zt_identity as *mut ZT_Identity);
            if ret == ZT_ResultCode_ZT_RESULT_FATAL_ERROR_OUT_OF_MEMORY {
                return Err(ZTError::OutOfMemory);
            }
        }
        Ok(Self {
            zt_identity: zt_identity,
            // TODO: fetch from identity
            _identity: String::from(""),
            _public_key: String::from(""),
            _private_key: String::from(""),
        })
    }

    // Load an identity from string
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

    // TODO: use to_string()
    unsafe fn to_string(&self) -> String {
        let mut buf = vec![0 as u8; 384];
        let ptr = ZT_Identity_toString(self.zt_identity as *mut ZT_Identity, true, buf.as_mut_ptr() as *mut c_char);
        let cstr = CStr::from_ptr(ptr).to_owned();
        return String::from(cstr.to_str().unwrap());
    }
}

impl Drop for Identity {
    fn drop(&mut self) {
        unsafe {
            ZT_Identity_delete(self.zt_identity as *mut ZT_Identity);
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate regex;

    use super::*;
    use regex::Regex;

    #[test]
    fn test_new_zt_identity() {
        let zt_identity: *const ZT_Identity = Identity::new_zt_identity().unwrap();
        assert!(zt_identity != std::ptr::null());
    }

    #[test]
    fn test_zt_identity_to_string() {
        let identity: Identity = Identity::new().unwrap();
        let id_str: String;
        unsafe {
            id_str = identity.to_string();
        }
        let re = Regex::new(r"^[a-z0-9]{10}:0:[a-z0-9]{128}:[a-z0-9]{128}$").unwrap();
        assert!(re.is_match(id_str.as_str()))
    }
}
