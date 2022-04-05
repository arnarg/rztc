extern crate ed25519_dalek;
extern crate x25519_dalek;
extern crate rand;
extern crate hex;
extern crate zerotier;

use rand::RngCore;
use rand::rngs::OsRng;
use zerotier::Address;
use failure::{Fallible, Fail};

#[derive(Debug, Fail)]
pub enum GenerateError {
    #[fail(display = "unable to generate valid identity")]
    TooManyRetries,
}

pub struct Identity {
    pub address: Address,
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
            let x_static = x25519_dalek::StaticSecret::from(x_secret.clone());
            let x_public = x25519_dalek::PublicKey::from(x_static.to_bytes());

            // Copy keys to final buffers
            secbuf[..32].copy_from_slice(&x_secret);
            secbuf[32..].copy_from_slice(&ed_keypair.secret.to_bytes());
            pubbuf[..32].copy_from_slice(&x_public.to_bytes());
            pubbuf[32..].copy_from_slice(&ed_keypair.public.to_bytes());

            let pubkey = zerotier::PublicKey::try_from(&pubbuf[..]).unwrap();
            if let Ok(address) = Address::try_from(&pubkey) {
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
        format!("{}:0:{}", serde_yaml::to_string(&self.address).unwrap(), hex::encode(self.public_key))
    }

    pub fn to_secret_string(&self) -> String {
        format!("{}:{}", self.to_public_string(), hex::encode(self.secret_key))
    }
}

#[cfg(test)]
mod tests {
    extern crate regex;

    use regex::Regex;
    use super::*;

    // TODO: write proper test
    #[test]
    fn test_generate() {
        // let buf: [u8; 32] = [0x04, 0x4f, 0x6a, 0xbd, 0x6a, 0xe6, 0x76, 0xae, 0x29, 0x6a, 0x23, 0xbb, 0x1b, 0x2f, 0x9f, 0xdf, 0xb6, 0xfb, 0xac, 0xdb, 0xf7, 0x88, 0xdc, 0x04, 0xda, 0xf2, 0x76, 0xa1, 0x04, 0xfc, 0x54, 0xcd];
        // let xkey = x25519_dalek::StaticSecret::from(buf);
        // let xkey = x25519_dalek::PublicKey::from(&xkey);
        // println!("{:x?}", xkey.to_bytes());

        let identity = Identity::generate().unwrap();
        println!("{}", identity.to_public_string());
        println!("{}", identity.to_secret_string());

        //println!("{}:0:{}:{}", hex::encode(identity.address.0), hex::encode(identity.public_key.into() as [u8; 64]), hex::encode(identity.secret_key.into() as [u8; 64]))
        // let id_str: str = identity.serialize();
        // // Make sure the secret string matches an expected pattern
        // let re = Regex::new(r"^[a-z0-9]{10}:0:[a-z0-9]{128}:[a-z0-9]{128}$").unwrap();
        // assert!(re.is_match(id_str));
        // // Make sure it's not filled with zeros, i.e. no data
        // let re2 = Regex::new(r"^0{10}:0:0{128}:0{128}$").unwrap();
        // assert!(!re2.is_match(id_str));
    }
}
