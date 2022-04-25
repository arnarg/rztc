use serde::{Serialize, Deserialize};
use std::str::FromStr;
use ipnetwork::Ipv4Network;
use failure::Fallible;
use sha2::Digest;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_secondary_port")]
    pub secondary_port: u16,
    pub identity_path: String,
    pub networks: Vec<Network>,
}

fn default_port() -> u16 { 9994 }
fn default_secondary_port() -> u16 { 29995 }

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Network {
    name: String,
    id: Option<String>,
    #[serde(default = "default_revision")]
    revision: u64,
    #[serde(default = "default_public")]
    public: bool,
    cidr: String,
    routes: Option<Vec<Route>>,
    #[serde(default = "default_broadcast")]
    broadcast: bool,
    #[serde(default = "default_multicast")]
    multicast_recipient_limit: u64,
    #[serde(default = "default_mtu")]
    mtu: u16,
    dns: Option<DNS>,
    members: Vec<Member>,
    rules: Option<Vec<Rule>>,
}

fn default_revision() -> u64 { 0 }
fn default_public() -> bool { false }
fn default_broadcast() -> bool { true }
fn default_multicast() -> u64 { 32 }
fn default_mtu() -> u16 { 2800 }

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
struct Route {
    destination: String,
    via: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
struct DNS {
    search_domain: String,
    server_address: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
struct Member {
    address: String,
    ip: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
struct Rule {
    #[serde(rename(deserialize = "type"))]
    rtype: String,
    #[serde(default = "default_not_or")]
    not: bool,
    #[serde(default = "default_not_or")]
    or: bool,
    #[serde(default)]
    ether_type: String,
}

fn default_not_or() -> bool {
    false
}

impl Member {
    /// Converts Member into zt::controller::Member
    ///
    /// If ip is not set it will use the the address to determine the ip address.
    /// It uses the opposite bytes from the netmask and ORs the end of the address
    /// and the network ip together.
    ///
    /// Example:
    ///  Network CIDR: 100.100.0.0/16
    ///  ZT Address:   aabbccddee
    ///  IP address:   100.100.dd.ee (100.100.221.238)
    ///
    /// ZeroTier addresses are already quite random data so collisions _shouldn't_ be
    /// too common on a fairly big and not so busy network.
    ///
    /// TODO: Check for broadcast IP.
    pub fn try_into_zt_member(self, network: &Ipv4Network) -> Fallible<zt::controller::Member> {
        let address: u64 = {
            let mut bytes = [0u8; 8];
            hex::decode_to_slice(self.address, &mut bytes[3..])?;
            u64::from_be_bytes(bytes)
        };

        let ip = match self.ip {
            Some(ip) => std::net::Ipv4Addr::from_str(ip.as_str())?,
            None => {
                let network_ip: u32 = network.ip().into();
                let mask: u32 = network.mask().into();

                let ip: u32 = (network_ip & mask) | (address as u32 & !mask);

                std::net::Ipv4Addr::from(ip)
            },
        };

        Ok(zt::controller::Member {
            address: address,
            ip: Ipv4Network::new(ip, network.prefix())?,
        })
    }
}

impl TryInto<zt::controller::Network> for Network {
    type Error = failure::Error;

    fn try_into(self) -> Result<zt::controller::Network, Self::Error> {
        let mut members: Vec<zt::controller::Member> = Vec::new();
        let network = Ipv4Network::from_str(self.cidr.as_str())?;

        for m in self.members {
            members.push(m.try_into_zt_member(&network)?);
        }

        let id = match self.id {
            Some(id) => {
                let mut bytes = [0u8; 4];
                hex::decode_to_slice(id, &mut bytes[1..])?;
                u32::from_be_bytes(bytes)
            },
            None => {
                let hash = sha2::Sha256::digest(self.name.as_str());
                u32::from_be_bytes(hash[..4].try_into()?) >> 8
            },
        };

        Ok(zt::controller::Network {
            name: self.name,
            id: id,
            prefix: network.prefix(),
            revision: self.revision,
            public: self.public,
            broadcast: self.broadcast,
            multicast_recipient_limit: self.multicast_recipient_limit,
            mtu: self.mtu,
            members: members,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_into_zt_member_without_ip() -> Fallible<()> {
        let network = Ipv4Network::from_str("100.100.0.0/20")?;

        let member = Member {
            address: "aabbccddee".to_string(),
            ip: None,
        };

        let zt_member = member.try_into_zt_member(&network)?;

        assert_eq!(zt_member.address, 0xaabbccddee);
        assert_eq!(zt_member.ip, ipnetwork::Ipv4Network::new(std::net::Ipv4Addr::new(100, 100, 13, 238), 20)?);

        Ok(())
    }

    #[test]
    fn test_into_zt_member_with_ip() -> Fallible<()> {
        let network = Ipv4Network::from_str("100.100.0.0/24")?;

        let member = Member {
            address: "aabbccddee".to_string(),
            ip: Some("100.100.0.10".to_string()),
        };

        let zt_member = member.try_into_zt_member(&network)?;

        assert_eq!(zt_member.address, 0xaabbccddee);
        assert_eq!(zt_member.ip, ipnetwork::Ipv4Network::new(std::net::Ipv4Addr::new(100, 100, 0, 10), 24)?);

        Ok(())
    }

    #[test]
    fn test_into_zt_network_without_id() -> Fallible<()> {
        let network = Network {
            name: "test-network".to_string(),
            id: None,
            revision: 0,
            public: false,
            cidr: "100.100.0.0/24".to_string(),
            routes: None,
            broadcast: true,
            multicast_recipient_limit: 32,
            mtu: 2800,
            members: vec![
                Member {
                    address: "aabbccddee".to_string(),
                    ip: None,
                },
                Member {
                    address: "a1b2c3d4e5".to_string(),
                    ip: Some("100.100.0.10".to_string()),
                },
            ],
            dns: None,
            rules: None,
        };

        let zt_network: zt::controller::Network = network.clone().try_into()?;

        assert_eq!(zt_network.name, network.name);
        assert_eq!(zt_network.id, 0x1841d1);

        Ok(())
    }

    #[test]
    fn test_into_zt_network_with_id() -> Fallible<()> {
        let network = Network {
            name: "test-network".to_string(),
            id: Some("abcdef".to_string()),
            revision: 0,
            public: false,
            cidr: "100.100.0.0/24".to_string(),
            routes: None,
            broadcast: true,
            multicast_recipient_limit: 32,
            mtu: 2800,
            members: vec![
                Member {
                    address: "aabbccddee".to_string(),
                    ip: None,
                },
                Member {
                    address: "a1b2c3d4e5".to_string(),
                    ip: Some("100.100.0.10".to_string()),
                },
            ],
            dns: None,
            rules: None,
        };

        let zt_network: zt::controller::Network = network.clone().try_into()?;

        assert_eq!(zt_network.name, network.name);
        assert_eq!(zt_network.id, 0xabcdef);

        Ok(())
    }
}
