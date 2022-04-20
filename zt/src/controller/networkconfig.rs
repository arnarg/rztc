use crate::controller::identity::Identity;
use crate::controller::certificate::CertificateOfMembership;
use crate::dictionary::Dictionary;
use failure::Fallible;

const NETWORKCONFIG_VERSION: u64 = 7;

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

pub struct NetworkConfig {
    pub name: String,
    pub nwid: u64,
    pub timestamp: i64,
    pub credential_time_max_delta: i64,
    pub rev: u64,
    pub issued_to: u64,
    pub flags: u64,
    pub mtu: u64,
    pub com: CertificateOfMembership,
}

impl NetworkConfig {
    pub fn serialize(&self) -> Fallible<Vec<u8>> {
        let mut dict = Dictionary::new();
        dict.set_u64(DICT_KEY_VERSION, NETWORKCONFIG_VERSION);
        dict.set_str(DICT_KEY_NAME, &self.name);
        dict.set_u64(DICT_KEY_NETWORK_ID, self.nwid);
        dict.set_u64(DICT_KEY_TIMESTAMP, self.timestamp.try_into()?);
        dict.set_u64(DICT_KEY_CREDENTIAL_TIME_MAX_DELTA, self.credential_time_max_delta.try_into()?);
        dict.set_u64(DICT_KEY_REVISION, self.rev);
        dict.set_u64(DICT_KEY_FLAGS, self.flags);
        // For whatever reason issued_to address should be a non-zero-padded hex string
        dict.set_str(DICT_KEY_ISSUED_TO, format!("{:x}", self.issued_to).as_str());
        dict.set_bytes(DICT_KEY_COM, &self.com.serialize()?);

        Ok(dict.finalize())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_network_config_serialize() -> Fallible<()> {
        let id = Identity {
            address: 589744919974,
            public: hex::decode("2ca7d749ec20a750b6189cf1f51a5f7db67bbed6218cbae506946c01e267cd05d6e4bd580af21231b7edd03eb04a086a43a14cfca67b19a1cc4484e5ad142034")?.try_into().unwrap(),
        };
        let nc = NetworkConfig {
            name: "test-network0".to_string(),
            nwid: 0x12345678654321,
            timestamp: 1650367222104,
            credential_time_max_delta: 123456,
            rev: 1,
            flags: 1,
            mtu: 9000,
            issued_to: id.address,
            com: CertificateOfMembership::new(
                1650367222104,
                123456,
                0x12345678654321,
                id.clone(),
            ),
        };

        unsafe {
            println!("{}", std::str::from_utf8_unchecked(&nc.serialize()?));
        }

        Ok(())
    }
}
