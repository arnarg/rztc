use super::Controller;
use zt_sys::controller::*;
use crate::dictionary::Dictionary;
use crate::controller::identity::Identity;
use ed25519_dalek::{Keypair, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH};

macro_rules! to_controller {
    ( $a:expr ) => {
        unsafe { &mut *($a as *mut Controller) }
    };
}

#[no_mangle]
pub extern "C" fn init_controller(
    _rztc_controller: *mut RZTC_Controller,
    controller: *mut std::os::raw::c_void,
    id: u64,
    kp: *const std::os::raw::c_void,
    kp_len: u64
) {
    // Recover the rust native Controller through the user pointer
    let c: &mut Controller = to_controller!(controller);
    // Cast keypair buffer to slice
    let buf = unsafe{ std::slice::from_raw_parts(kp as *const u8, kp_len as usize) };

    // The buffer we receive contains both public keys and private keys as represented
    // by zerotier identity. It contains both curve25519 public and secret key and
    // ed25519 public and secret key. ed25519 is used for signing so we only care about
    // that here.
    // First 64 bytes are public keys and the second 64 bytes are private keys. Each
    // half contains first curve25519 key and then ed25519 key, each 32 bytes.
    //
    // |--       Public       --|--      Private       --|
    // |--   32   --|--   32  --|--   32   --|--   32  --|
    // | curve25519 |  ed25519  | curve25519 |  ed25519  |
    //
    // ed25519_dalek::Keypair::from_bytes expects first 32 bytes secret key and then
    // 32 bytes public key.
    //
    // TODO: Validate data
    let mut keys = [0u8; PUBLIC_KEY_LENGTH + SECRET_KEY_LENGTH];
    keys[..SECRET_KEY_LENGTH].copy_from_slice(&buf[96..128]);
    keys[SECRET_KEY_LENGTH..].copy_from_slice(&buf[32..64]);
    let keypair = Keypair::from_bytes(&keys[..]).unwrap();
    c.set_keypair(id, keypair);
}

#[no_mangle]
pub extern "C" fn on_network_request(
    _rztc_controller: *mut RZTC_Controller,
    controller: *mut std::os::raw::c_void,
    nwid: u64,
    _sockaddr: *const libc::sockaddr_storage,
    packet_id: u64,
    identity: u64,
    public_key: *const std::os::raw::c_void,
    metadata_dict: *const std::os::raw::c_void,
    max_len: u64,
) {
    // Recover the rust native Controller through the user pointer
    let c: &mut Controller = to_controller!(controller);
    // Cast metadata_dict to slice
    let buf = unsafe{ std::slice::from_raw_parts(metadata_dict as *const u8, max_len as usize) };
    let index: usize = match buf.iter().position(|x| *x == 0) {
        Some(i) => i,
        None => max_len as usize,
    };
    let dict = Dictionary::from(Vec::from(&buf[..index]));
    // Create rust native identity struct
    let pub_buf = unsafe{
        let mut buf = [0u8; 64];
        buf[..].copy_from_slice(std::slice::from_raw_parts(public_key as *const u8, 64 as usize));
        buf
    };
    let id = Identity {
        address: identity,
        public: pub_buf.clone(),
    };


    c.on_request(nwid, packet_id, id, dict);
}
