extern crate ed25519_dalek;
extern crate x25519_dalek;
extern crate rand;
extern crate hex;
extern crate zerotier;

use rand::rngs::OsRng;
use zerotier::{Identity, SecretKey};
use failure::{Fallible, Fail};

#[derive(Debug, Fail)]
pub enum GenerateError {
    #[fail(display = "unable to generate valid identity")]
    TooManyRetries,
}

pub trait IdentityGenerator {
    fn generate() -> Fallible<Identity>;
}

impl IdentityGenerator for Identity {
    fn generate() -> Fallible<Identity> {
        for _ in 1..100 {
            // Generate ed25519 keypair
            let ed_keypair = ed25519_dalek::Keypair::generate(&mut OsRng);
            // Generate x25519 secret
            let x_secret = x25519_dalek::StaticSecret::new(OsRng);
            
            // Pack private key in a single 64 byte array expected by
            // zerotier Identity
            let priv_key = &mut [0 as u8; 64];
            priv_key[..32].copy_from_slice(&x_secret.to_bytes());
            priv_key[32..64].copy_from_slice(&ed_keypair.secret.to_bytes());

            let secret_key = SecretKey::from(*priv_key);
            if let Ok(identity) = Identity::try_from(secret_key) {
                return Ok(identity);
            }
        }
        Err(GenerateError::TooManyRetries.into())
    }
}

#[cfg(test)]
mod tests {
    extern crate regex;

    use regex::Regex;
    use zerotier::Identity;
    use crate::identity::IdentityGenerator;

    // TODO: write proper test
    #[test]
    fn test_generate() {
        let identity = Identity::generate().unwrap();
        println!("{:?}", identity.address);
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
