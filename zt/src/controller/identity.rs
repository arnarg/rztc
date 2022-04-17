use sha2::Digest;

#[derive(Debug, Clone)]
pub struct Identity {
    pub address: u64,
    pub public: [u8; 64],
}

impl Identity {
    pub fn public_key_hash(&self) -> [u8; 48] {
        let addr_buf = &self.address.to_be_bytes()[3..];
        let mut hasher = sha2::Sha384::new();
        hasher.update(addr_buf);
        hasher.update(self.public);
        let mut buf = [0u8; 48];
        buf[..].copy_from_slice(&hasher.finalize().as_slice());
        buf
    }
}

// TODO: write tests pls!
