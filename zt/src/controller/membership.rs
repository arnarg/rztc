use crate::controller::identity::Identity;
use crate::controller::ZeroTierSigner;
use serde::{Serialize, Deserialize};
use failure::Fallible;

#[derive(Serialize, Deserialize, Debug)]
enum QualifierId {
    Timestamp = 0,
    NetworkId = 1,
    IssuedTo = 2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Qualifier {
    id: u64,
    value: u64,
    max_delta: u64,
}

#[derive(Debug, Clone)]
pub struct CertificateOfMembership {
    qualifiers: Vec<Qualifier>,
    signer: u64,
    signature: [u8; 96],
}

impl CertificateOfMembership {
    pub fn new(ts: u64, delta: u64, nwid: u64, identity: &Identity) -> Self {
        let mut qualifiers = Vec::new();
        qualifiers.push(Qualifier {
            id: QualifierId::Timestamp as u64,
            value: ts,
            max_delta: delta,
        });
        qualifiers.push(Qualifier {
            id: QualifierId::NetworkId as u64,
            value: nwid,
            max_delta: 0,
        });
        qualifiers.push(Qualifier {
            id: QualifierId::IssuedTo as u64,
            value: identity.address,
            max_delta: 0xffffffffffffffff,
        });

        let identity_hash = identity.public_key_hash();
        let mut identity_iter = identity_hash.chunks(8);
        for i in 3..7 {
            match identity_iter.next() {
                Some(c) => {
                    qualifiers.push(Qualifier {
                        id: i,
                        value: u64::from_be_bytes(c.try_into().unwrap()),
                        max_delta: 0xffffffffffffffff,
                    });
                },
                None => break,
            }
        }

        Self {
            qualifiers: qualifiers,
            signer: 0,
            signature: [0u8; 96],
        }
    }

    pub fn serialize(&self) -> Fallible<Vec<u8>> {
        // In ZeroTier CertificateOfMembership is serialized like so:
        // -----------------------------
        // | u8  | 1                   | // always 1
        // -----------------------------
        // | u16 | num qualifiers      |
        // -----------------------------
        // | u64 | qualifier id        | -
        // ----------------------------- |
        // | u64 | qualifier val       | |- repeats n times
        // ----------------------------- |
        // | u64 | qualifier max delta | -
        // -----------------------------
        // | u64 | signer identity     |
        // -----------------------------
        // | u8 * 96 | signature       |
        // -----------------------------
        let mut out = Vec::new();
        out.push(1);
        out.append(&mut u16::to_be_bytes(self.qualifiers.len() as u16).to_vec());
        for q in &self.qualifiers {
            out.append(&mut u64::to_be_bytes(q.id).to_vec());
            out.append(&mut u64::to_be_bytes(q.value).to_vec());
            out.append(&mut u64::to_be_bytes(q.max_delta).to_vec());
        }
        out.append(&mut u64::to_be_bytes(self.signer)[3..].to_vec());
        if self.signer != 0 {
            out.append(&mut self.signature.to_vec());
        }
        Ok(out)
    }

    pub fn sign(&mut self, identity: u64, signer: &dyn ZeroTierSigner) -> Fallible<()> {
        let mut buf: Vec<u8> = Vec::new();
        for q in &self.qualifiers {
            buf.append(&mut u64::to_be_bytes(q.id).to_vec());
            buf.append(&mut u64::to_be_bytes(q.value).to_vec());
            buf.append(&mut u64::to_be_bytes(q.max_delta).to_vec());
        }

        self.signature.copy_from_slice(&signer.sign(&buf)?);
        self.signer = identity;

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use sha2::Digest;
    use ed25519_dalek::{Keypair, Signer, KEYPAIR_LENGTH};

    struct TestSigner(ed25519_dalek::Keypair);

    impl ZeroTierSigner for TestSigner {
        fn sign(&self, data: &[u8]) -> Fallible<[u8; 96]> {
            let mut signature = [0u8; 96];
            let digest = &sha2::Sha512::digest(data)[..32];
            let signed = self.0.sign(&digest).to_bytes();
            signature[..KEYPAIR_LENGTH].copy_from_slice(&signed);
            signature[KEYPAIR_LENGTH..].copy_from_slice(digest);
            Ok(signature)
        }
    }

    #[test]
    fn test_certificate_of_membership() -> Fallible<()> {
        let mut com = CertificateOfMembership::new(
            1650367222104,
            123456,
            5124095572525857, // 0x12345678654321
            &Identity {
                address: 589744919974,
                public: hex::decode("2ca7d749ec20a750b6189cf1f51a5f7db67bbed6218cbae506946c01e267cd05d6e4bd580af21231b7edd03eb04a086a43a14cfca67b19a1cc4484e5ad142034")?.try_into().unwrap(),
            },
        );


        let expect = "010007000000000000000000000180418d5158000000000001e2400000000000000001001234567865432100000000000000000000000000000002000000894f8955a6ffffffffffffffff0000000000000003c479b54bc47ea678ffffffffffffffff000000000000000432998ed6255eadf2ffffffffffffffff00000000000000053a43d68294a4bdffffffffffffffffff0000000000000006566323366e1bc33effffffffffffffffaabbccddeebce1c9e0120816faf3ac10b0eef4048614bec556ef5ed0deb7bc513ec5fdff7a6f89a6e0aeb50340defae92cf16595929ffed35a7b4d5fe4fd11d494afb5a30e662ab1c84a83f53211b6f0ccf764017b871670eea12e07a1d7888c9e60c29b8e";

        let keypair_hex = "6b7ec6bebb42159e30f8fe843c1e2928372a8787a098d39f41568282a0c89637f82dc186e19f50b01e7fa93637683919de6e4be7df1a3404a9f21ba16d273f94";
        let signer = TestSigner(Keypair::from_bytes(&hex::decode(keypair_hex)?)?);

        com.sign(0xaabbccddee, &signer)?;

        assert_eq!(com.serialize()?, hex::decode(expect)?);

        Ok(())
    }
}
