#![allow(dead_code)]

use crate::controller::identity::Identity;
use crate::controller::certificate::CertificateOfMembership;
use crate::dictionary::Dictionary;
use std::time::{SystemTime, UNIX_EPOCH};
use ed25519_dalek::Keypair;
use failure::Fallible;

const NETWORKCONFIG_VERSION: u64 = 7;

const NETWORKCONFIG_DEFAULT_CREDENTIAL_TIME_MAX_DELTA: u64 = 7200000;

// Keys used for serializing network config to dictionary
const DICT_KEY_VERSION: &str = "v";
const DICT_KEY_NETWORK_ID: &str = "nwid";
const DICT_KEY_TIMESTAMP: &str = "ts";
const DICT_KEY_REVISION: &str = "r";
const DICT_KEY_ISSUED_TO: &str = "id";
const DICT_KEY_REMOTE_TRACE_TARGET: &str = "tt";
const DICT_KEY_REMOTE_TRACE_LEVEL: &str = "tl";
const DICT_KEY_FLAGS: &str = "f";
const DICT_KEY_MULTICAST_LIMIT: &str = "ml";
const DICT_KEY_TYPE: &str = "t";
const DICT_KEY_NAME: &str = "n";
const DICT_KEY_MTU: &str = "mtu";
const DICT_KEY_CREDENTIAL_TIME_MAX_DELTA: &str = "ctmd";
const DICT_KEY_COM: &str = "C";
const DICT_KEY_SPECIALISTS: &str = "S";
const DICT_KEY_ROUTES: &str = "RT";
const DICT_KEY_STATIC_IPS: &str = "I";
const DICT_KEY_RULES: &str = "R";
const DICT_KEY_CAPABILITIES: &str = "CAP";
const DICT_KEY_TAGS: &str = "TAG";
const DICT_KEY_CERTIFICATES_OF_OWNERSHIP: &str = "COO";
const DICT_KEY_DNS: &str = "DNS";
const DICT_KEY_SSO_ENABLED: &str = "ssoe";
const DICT_KEY_SSO_VERSION: &str = "ssov";
const DICT_KEY_AUTHENTICATION_URL: &str = "aurl";
const DICT_KEY_AUTHENTICATION_EXPIRY_TIME: &str = "aexpt";
const DICT_KEY_ISSUER_URL: &str = "iurl";
const DICT_KEY_CENTRAL_ENDPOINT_URL: &str = "ssoce";
const DICT_KEY_NONCE: &str = "sson";
const DICT_KEY_STATE: &str = "ssos";
const DICT_KEY_CLIENT_ID: &str = "ssocid";

#[allow(dead_code)]
pub enum NetworkType {
    Private = 0,
    Public = 1,
}

#[allow(dead_code)]
pub enum TraceLevel {
    Normal = 0,
    Verbose = 10,
    Rules = 15,
    Debug = 20,
    Insane = 30, // That is what they call it in the ZeroTierOne source code :)
}

pub struct NetworkConfig {
    pub name: String,
    pub nwid: u64,
    pub timestamp: i64,
    pub credential_time_max_delta: u64,
    pub rev: u64,
    pub multicast_limit: u64,
    pub network_type: u64,
    pub issued_to: u64,
    pub trace_target: u64,
    pub trace_level: u64,
    pub flags: u64,
    pub mtu: u64,
    pub com: CertificateOfMembership,
}

impl NetworkConfig {
    pub fn new(name: &str, nwid: u64, issued_to: &Identity, rev: u64) -> Fallible<Self> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let now: i64 = now.as_millis().try_into()?;
        Ok(Self {
            name: name.to_string(),
            nwid: nwid,
            timestamp: now,
            credential_time_max_delta: NETWORKCONFIG_DEFAULT_CREDENTIAL_TIME_MAX_DELTA,
            network_type: NetworkType::Private as u64,
            multicast_limit: 0,
            rev: rev,
            mtu: 2800,
            flags: 0,
            issued_to: issued_to.address,
            trace_target: 0,
            trace_level: TraceLevel::Normal as u64,
            com: CertificateOfMembership::new(
                now as u64,
                NETWORKCONFIG_DEFAULT_CREDENTIAL_TIME_MAX_DELTA,
                nwid,
                &issued_to,
            ),
        })
    }

    pub fn serialize(&self) -> Fallible<Vec<u8>> {
        let mut dict = Dictionary::new();
        dict.set_u64(DICT_KEY_VERSION, NETWORKCONFIG_VERSION);
        dict.set_u64(DICT_KEY_NETWORK_ID, self.nwid);
        dict.set_u64(DICT_KEY_TIMESTAMP, self.timestamp.try_into()?);
        dict.set_u64(DICT_KEY_CREDENTIAL_TIME_MAX_DELTA, self.credential_time_max_delta.try_into()?);
        dict.set_u64(DICT_KEY_REVISION, self.rev);
        // For whatever reason issued_to address should be a non-zero-padded hex string
        dict.set_str(DICT_KEY_ISSUED_TO, format!("{:010x}", self.issued_to).as_str());
        dict.set_str(DICT_KEY_REMOTE_TRACE_TARGET, format!("{:010x}", self.trace_target).as_str());
        dict.set_u64(DICT_KEY_REMOTE_TRACE_LEVEL, self.trace_level);
        dict.set_u64(DICT_KEY_FLAGS, self.flags);
        dict.set_u64(DICT_KEY_MULTICAST_LIMIT, self.multicast_limit);
        dict.set_u64(DICT_KEY_TYPE, self.network_type);
        dict.set_str(DICT_KEY_NAME, &self.name);
        dict.set_u64(DICT_KEY_MTU, self.mtu);
        dict.set_bytes(DICT_KEY_COM, &self.com.serialize()?);

        // Temporary hardcoded until implemented
        dict.set_bytes(DICT_KEY_RULES, &[1, 0]); // accept all
        dict.set_u64(DICT_KEY_SSO_VERSION, 0);
        dict.set_bool(DICT_KEY_SSO_ENABLED, false);
        dict.set_bytes(DICT_KEY_DNS, &[0u8; 132]);

        Ok(dict.finalize())
    }

    pub fn sign(&mut self, signing_id: u64, signing_keypair: &Keypair) {
        self.com.sign(signing_id, signing_keypair);
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    // TODO: implement actual test
    #[test]
    fn test_network_config_serialize() -> Fallible<()> {
        let id = Identity {
            address: 589744919974,
            public: hex::decode("2ca7d749ec20a750b6189cf1f51a5f7db67bbed6218cbae506946c01e267cd05d6e4bd580af21231b7edd03eb04a086a43a14cfca67b19a1cc4484e5ad142034")?.try_into().unwrap(),
        };
        let nc = NetworkConfig::new("test-network0", 0x12345678654321, &id, 1)?;

        unsafe {
            println!("{}", std::str::from_utf8_unchecked(&nc.serialize()?));
        }

        Ok(())
    }
}