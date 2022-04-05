extern crate ed25519_dalek;
extern crate x25519_dalek;
extern crate rand;
extern crate hex;
extern crate zerotier;

use rand::RngCore;
use rand::rngs::OsRng;
use failure::{Fallible, Fail, Error};
use std::fmt;
use hex::FromHex;
use crate::core::ConfigurationProvider;

pub use zerotier::InternalError;

#[derive(Debug, Fail)]
pub enum GenerateError {
    #[fail(display = "unable to generate valid identity")]
    TooManyRetries,
}

pub struct Identity {
    pub address: zerotier::Address,
    pub public_key: [u8; 64],
    secret_key: [u8; 64],
}

impl Identity {
    pub fn generate() -> Fallible<Identity> {
        for _ in 0..100 {
            let mut pubbuf = [0u8; 64];
            let mut secbuf = [0u8; 64];

            // Generate ed25519 keypair
            let ed_keypair = ed25519_dalek::Keypair::generate(&mut OsRng);
            // Generate random x25519 secret to first half of buffer
            let mut x_secret = [0u8; 32];
            OsRng.fill_bytes(&mut x_secret);
            // Get x25519 public key
            let x_public = x25519_dalek::PublicKey::from(
                    x25519_dalek::StaticSecret::from(x_secret.clone()).to_bytes()
                );

            // Copy keys to final buffers
            secbuf[..32].copy_from_slice(&x_secret);
            secbuf[32..].copy_from_slice(&ed_keypair.secret.to_bytes());
            pubbuf[..32].copy_from_slice(&x_public.to_bytes());
            pubbuf[32..].copy_from_slice(&ed_keypair.public.to_bytes());

            if let Ok(address) = buf_to_address(&pubbuf[..]) {
                return Ok(Self {
                    address: address,
                    public_key: pubbuf,
                    secret_key: secbuf,
                });
            }
        }
        Err(GenerateError::TooManyRetries.into())
    }

    pub fn to_public_string(&self) -> String {
        format!("{}:0:{}", self.address, hex::encode(self.public_key))
    }

    pub fn to_secret_string(&self) -> String {
        format!("{}:{}", self.to_public_string(), hex::encode(self.secret_key))
    }
}

/// Tries to derive identity from str. Throws
/// [`InternalError`](enum.InternalError.html) for invalid addresses.
impl TryFrom<&str> for Identity {
    type Error = Error;

    fn try_from(identity: &str) -> Fallible<Self> {
        let split: Vec<&str> = identity.split(':').collect();
        let (address, public_key, secret_key) = match &split[..] {
            [address, "0", public_key, secret_key] => (address, public_key, secret_key),
            _ => return Err(InternalError::MalformedIdentity.into())
        };

        Ok(Self {
            address: zerotier::Address::try_from(hex::decode(address)?.as_slice())?,
            public_key: <[u8; 64]>::from_hex(public_key)?,
            secret_key: <[u8; 64]>::from_hex(secret_key)?,
        })
    }
}

// Implement ConfigurationProvider so we can pass Identity straight
// to a Node. Set operations are no-op.
impl ConfigurationProvider for Identity {
    fn get_public_identity(&self) -> String {
        self.to_public_string()
    }

    fn get_secret_identity(&self) -> String {
        self.to_secret_string()
    }

    fn set_public_identity(&self, _public_identity: String) -> bool {
        false
    }

    fn set_secret_identity(&self, _secret_identity: String) -> bool {
        false
    }
}

impl fmt::Display for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.address)
    }
}

fn buf_to_address(public_key: &[u8]) -> Fallible<zerotier::Address> {
    let pubkey = zerotier::PublicKey::try_from(public_key)?;
    zerotier::Address::try_from(&pubkey)
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: write proper test
    #[test]
    fn test_identity() {
        let identity_str = "666e787206:0:b0740d0eb870517d835d5d1080bebe84c3097acebcf737f4872c574f2716ca534f9abf533c0ff40cbe028aa7a5774762a77e9e85b4cddf67d892ef59234704f7:b7740d0eb870517d835d5d1080bebe84c3097acebcf737f4872c574f2716cad39272c4ef4e23f0d7414b1c181a86d9f66a38f51fda0e4c252abf0ba22e67371a";
        let identity = Identity::try_from(identity_str).unwrap();

        assert_eq!(format!("{}", identity), "666e787206");

        assert_eq!(identity.to_public_string(), "666e787206:0:b0740d0eb870517d835d5d1080bebe84c3097acebcf737f4872c574f2716ca534f9abf533c0ff40cbe028aa7a5774762a77e9e85b4cddf67d892ef59234704f7");

        assert_eq!(identity.to_secret_string(), identity_str);
    }
}
