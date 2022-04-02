#![allow(non_upper_case_globals)]

extern crate zt_sys;
extern crate libc;
extern crate sha2;
extern crate sodiumoxide;

use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use sha2::{Sha512, Digest};
use sodiumoxide::crypto::stream::salsa20::*;
use byteorder::{ByteOrder, BigEndian};
use crate::core::ZTError;
use zt_sys::*;

// Size of has computational buffer, taken from go-ztidentity
const IDENTITY_GEN_MEM_SIZE: usize = 2097152;

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

    // Re-implementation of computeZeroTierIdentityMemoryHardHash in go-ztidentity
    fn compute_hash(pub_key: &[u8]) -> Vec<u8> {
        // Compute sha512 checksum of public key
        let mut s512 = Sha512::digest(pub_key);

        // Setup computation buffer of size IDENTITY_GEN_MEM_SIZE
        let mut gen_mem = vec![0 as u8; IDENTITY_GEN_MEM_SIZE];

        // Derive key and nonce for salsa20 from sha512 hash
        let mut s20key = [0 as u8; 32];
        let mut s20nonce = [0 as u8; 8];
        s20key.copy_from_slice(&s512.as_slice()[..32]);
        s20nonce.copy_from_slice(&s512.as_slice()[32..40]);
        let n = Nonce::from_slice(&s20nonce).unwrap();
        let k = Key::from_slice(&s20key).unwrap();

        // Encrypt first 64 bytes of computation buffer using salsa20 streaming xor cipher
        stream_xor_inplace(&mut gen_mem.as_mut_slice()[..64], &n, &k);

        // Set up block counter
        let mut s20ctri: u64 = 1;

        // Encrypt rest of computation buffer using salsa20 streaming xor cipher
        let mut i = 64;
        while i < IDENTITY_GEN_MEM_SIZE {
            let tmp = stream_xor_ic(&mut gen_mem.as_mut_slice()[i-64..i], &n, s20ctri, &k);
            let part = &mut gen_mem.as_mut_slice()[i..i+64];
            part.copy_from_slice(tmp.as_slice());
            s20ctri += 1;
            i += 64;
        }

        // ??
        let mut tmp = [0 as u8; 8];
        i = 0;
        while i < IDENTITY_GEN_MEM_SIZE {
            let mut gms = &gen_mem.as_mut_slice()[i..];
            let idx1: usize = ((BigEndian::read_u64(&mut gms)&7) * 8).try_into().unwrap();
            i += 8;
            gms = &gen_mem.as_mut_slice()[i..];
            let idx2: usize = ((BigEndian::read_u64(&mut gms) as usize % (IDENTITY_GEN_MEM_SIZE/8)) * 8).try_into().unwrap();
            i += 8;
            let gm = &mut gen_mem.as_mut_slice()[idx2..idx2+8];
            let d = &mut s512.as_mut_slice()[idx1..idx1+8];
            tmp.copy_from_slice(gm);
            gm.copy_from_slice(d);
            d.copy_from_slice(&tmp);
            stream_xor_ic_inplace(&mut s512.as_mut_slice(), &n, s20ctri, &k);
            s20ctri += 1;
        }

        return Vec::from(s512.clone().as_slice());
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
    extern crate hex_literal;

    use super::*;
    use regex::Regex;
    use hex_literal::hex;

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

    #[test]
    fn test_compute_hash() {
        let pub_key = hex!("297602c1e90bccf4ca8db4777c92145d382e53f2908cff9a7781e84ec80d0049bd0cf9bd4e011ce65bf235bd99e835c00c35a390e6335fc52aca49b88a3ab1a9");
        let hash = Identity::compute_hash(&pub_key[..]);

        let expected = hex!("00b672c31aab3a49335859ff98c20ad904a90206595363663176d5b38dc2bc58d18fc21b548bf4cc41045d6728c525981ef76e600b9bc80b4342a92113984162");
        assert_eq!(hash, Vec::from(&expected[..]));
    }
}
