use sha2::Digest;

#[derive(Debug, Clone)]
pub struct Identity {
    pub address: u64,
    pub public: [u8; 64],
}

impl Identity {
    pub fn public_key_hash(&self) -> [u8; 48] {
        let addr_buf = &self.address.to_be_bytes()[3..];
        let mut buf = [0u8; 48];
        buf[..].copy_from_slice(
            sha2::Sha384::new()
                .chain(addr_buf)
                .chain(self.public)
                .finalize().as_slice()
        );
        buf
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use failure::Fallible;

    #[test]
    fn test_public_key_hash() -> Fallible<()> {
        let id = Identity {
            address: 589744919974, // 894f8955a6
            public: hex::decode("2ca7d749ec20a750b6189cf1f51a5f7db67bbed6218cbae506946c01e267cd05d6e4bd580af21231b7edd03eb04a086a43a14cfca67b19a1cc4484e5ad142034")?.try_into().unwrap(),
        };

        let expected = "c479b54bc47ea67832998ed6255eadf23a43d68294a4bdff566323366e1bc33e713c7abc68fdba16345c53ff53fa67b0";

        assert_eq!(id.public_key_hash(), hex::decode(expected)?.as_slice());

        Ok(())
    }
}
