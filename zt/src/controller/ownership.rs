use crate::controller::identity::Identity;
use crate::controller::ZeroTierSigner;
use std::net::{IpAddr, IpAddr::V4, IpAddr::V6};
use failure::Fallible;

enum ThingType {
    Ipv4Address = 2,
    Ipv6Address = 3,
}

#[derive(Debug, Clone)]
pub struct CertificateOfOwnership {
    nwid: u64,
    timestamp: u64,
    flags: u64,
    id: u32,
    ips: Vec<IpAddr>,
    issued_to: u64,
    signer: u64,
    signature: [u8; 96],
}

impl CertificateOfOwnership {
    pub fn new(ts: u64, nwid: u64, identity: &Identity, id: u32) -> Self {
        Self {
            nwid: nwid,
            timestamp: ts,
            flags: 0, // This doesn't really seem to be used but
                      // needs to be serialized
            id: id,
            issued_to: identity.address,
            ips: Vec::new(),
            signer: 0,
            signature: [0u8; 96],
        }
    }

    pub fn add_ip(&mut self, ip: &IpAddr) {
        self.ips.push(ip.clone());
    }

    pub fn serialize(&self, with_signature: bool) -> Fallible<Vec<u8>> {
        let mut out = Vec::new();
        out.append(&mut u64::to_be_bytes(self.nwid).to_vec());
        out.append(&mut u64::to_be_bytes(self.timestamp).to_vec());
        out.append(&mut u64::to_be_bytes(self.flags).to_vec());
        out.append(&mut u32::to_be_bytes(self.id).to_vec());

        out.append(&mut u16::to_be_bytes(self.ips.len() as u16).to_vec());
        for ip in &self.ips {
            let mut buf = [0u8; 16];
            match ip {
                V4(ip) => {
                    out.append(&mut u8::to_be_bytes(ThingType::Ipv4Address as u8).to_vec());
                    buf[..4].copy_from_slice(&ip.octets());
                },
                V6(ip) => {
                    out.append(&mut u8::to_be_bytes(ThingType::Ipv6Address as u8).to_vec());
                    buf[..16].copy_from_slice(&ip.octets());
                },
            }
            out.append(&mut buf.to_vec());
        }

        out.append(&mut u64::to_be_bytes(self.issued_to)[3..].to_vec());
        out.append(&mut u64::to_be_bytes(self.signer)[3..].to_vec());

        if with_signature {
            out.push(1); // 1 == Ed25519
            out.append(&mut u16::to_be_bytes(self.signature.len() as u16).to_vec());
            out.append(&mut self.signature.to_vec());
        }

        out.append(&mut u16::to_be_bytes(0).to_vec()); // Length of additional fields,
                                                       // currently zero

        Ok(out)
    }

    pub fn sign(&mut self, identity: u64, signer: &dyn ZeroTierSigner) -> Fallible<()> {
        self.signer = identity;
        let mut buf = Vec::new();
        buf.append(&mut u64::to_be_bytes(0x7f7f7f7f7f7f7f7f).to_vec());
        buf.append(&mut self.serialize(false)?);
        buf.append(&mut u64::to_be_bytes(0x7f7f7f7f7f7f7f7f).to_vec());

        self.signature.copy_from_slice(&signer.sign(&buf)?);

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use sha2::Digest;
    use ed25519_dalek::{Keypair, Signer, KEYPAIR_LENGTH};
    use std::net::{Ipv4Addr, Ipv6Addr};

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
    fn test_certificate_of_ownership() -> Fallible<()> {
        let mut coo = CertificateOfOwnership::new(
            1650367222104,
            0xaabbccddee123456,
            &Identity {
                address: 0xaabbccddee,
                public: hex::decode("2ca7d749ec20a750b6189cf1f51a5f7db67bbed6218cbae506946c01e267cd05d6e4bd580af21231b7edd03eb04a086a43a14cfca67b19a1cc4484e5ad142034")?.try_into().unwrap(),
            },
            1,
        );

        coo.add_ip(&IpAddr::V4(Ipv4Addr::new(100, 100, 0, 50)));
        //coo.add_ip(&IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)));

        let keypair_hex = "6b7ec6bebb42159e30f8fe843c1e2928372a8787a098d39f41568282a0c89637f82dc186e19f50b01e7fa93637683919de6e4be7df1a3404a9f21ba16d273f94";
        let signer = TestSigner(Keypair::from_bytes(&hex::decode(keypair_hex)?)?);

        coo.sign(0xabcdef1234, &signer)?;

        println!("{}", hex::encode(coo.serialize(true)?));

        Ok(())
    }
}
