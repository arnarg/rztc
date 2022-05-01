use super::error::*;
use zt_sys::*;
use std::str::FromStr;
use failure::Fallible;
use std::collections::BTreeMap;

/*
 * ACTIONS
 */
const ACTION_DROP: isize = 0;
const ACTION_ACCEPT: isize = 1;
const ACTION_TEE: isize = 2;
const ACTION_WATCH: isize = 3;
const ACTION_REDIRECT: isize = 4;
const ACTION_BREAK: isize = 5;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ActionType {
    Accept = ACTION_ACCEPT,
    Drop = ACTION_DROP,
    Break = ACTION_BREAK,
    Tee = ACTION_TEE,
    Watch = ACTION_WATCH,
    Redirect = ACTION_REDIRECT,
}

impl FromStr for ActionType {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<ActionType, Self::Err> {
        match input {
            "ACTION_DROP"      => Ok(ActionType::Drop),
            "ACTION_ACCEPT"    => Ok(ActionType::Accept),
            "ACTION_TEE"       => Ok(ActionType::Tee),
            "ACTION_WATCH"     => Ok(ActionType::Watch),
            "ACTION_REDIRECT"  => Ok(ActionType::Redirect),
            "ACTION_BREAK"     => Ok(ActionType::Break),
            _                  => Err(ParseError::NotFound),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleAction(ActionType);
impl SimpleAction {
    pub fn new(action_type: ActionType) -> Fallible<Self> {
        let t = match action_type {
            ActionType::Accept |
            ActionType::Drop |
            ActionType::Break => action_type,
            _                 => return Err(ParseError::NotFound.into()),
        };

        Ok(Self(t))
    }

    fn serialize(&self) -> Fallible<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(self.0 as u8);
        buf.push(0);
        Ok(buf)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComplexAction {
    type_id: ActionType,
    address: u64,
    flags: u32,
    length: u16,
}
impl ComplexAction {
    pub fn new(data: &BTreeMap<String, String>) -> Fallible<Self> {
        let type_id = match data.get("type") {
            Some(id) => ActionType::from_str(id)?,
            None     => return Err(ParseError::NotFound.into()),
        };

        let address = match data.get("address") {
            Some(address) => {
                let mut buf = [0u8; 8];
                buf[3..].copy_from_slice(&hex::decode(address)?[..]);
                u64::from_be_bytes(buf)
            },
            None => return Err(ParseError::NotFound.into()),
        };

        let flags = match data.get("flags") {
            Some(flags) => u32::from_str(flags)?,
            None        => 0,
        };

        let length = match data.get("length") {
            Some(length) => u16::from_str(length)?,
            None         => {
                // Not used for redirect
                if type_id != ActionType::Redirect {
                    return Err(ParseError::NotFound.into());
                }
                0
            }
        };

        Ok(Self {
            type_id: type_id,
            address: address,
            flags: flags,
            length: length,
        })
    }

    fn serialize(&self) -> Fallible<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(self.type_id as u8);
        buf.push(14); // 64+32+16/8 = 14
        buf.append(&mut u64::to_be_bytes(self.address).to_vec());
        buf.append(&mut u32::to_be_bytes(self.flags).to_vec());
        buf.append(&mut u16::to_be_bytes(self.length).to_vec());
        Ok(buf)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuleAction {
    Accept(SimpleAction),
    Drop(SimpleAction),
    Break(SimpleAction),
    Tee(ComplexAction),
    Redirect(ComplexAction),
    Watch(ComplexAction),
}

impl TryFrom<BTreeMap<String, String>> for RuleAction {
    type Error = failure::Error;

    fn try_from(data: BTreeMap<String, String>) -> Result<Self, Self::Error> {
        let type_id = match data.get("type") {
            Some(id) => id,
            None     => return Err(ParseError::NotFound.into()),
        };

        match ActionType::from_str(type_id) {
            Ok(ActionType::Accept)   => Ok(Self::Accept(SimpleAction::new(ActionType::Accept)?)),
            Ok(ActionType::Drop)     => Ok(Self::Drop(SimpleAction::new(ActionType::Drop)?)),
            Ok(ActionType::Break)    => Ok(Self::Break(SimpleAction::new(ActionType::Break)?)),
            Ok(ActionType::Tee)      => Ok(Self::Tee(ComplexAction::new(&data)?)),
            Ok(ActionType::Redirect) => Ok(Self::Redirect(ComplexAction::new(&data)?)),
            Ok(ActionType::Watch)    => Ok(Self::Watch(ComplexAction::new(&data)?)),
            Err(_)                   => Err(ParseError::NotFound.into())
        }
    }
}

/*
 * MATCHES
 */
const MATCH_ZT_SOURCE: isize = 24;
const MATCH_ZT_DEST: isize = 25;
const MATCH_VLAN_ID: isize = 26;
const MATCH_VLAN_PCP: isize = 27;
const MATCH_VLAN_DEI: isize = 28;
const MATCH_MAC_SOURCE: isize = 29;
const MATCH_MAC_DEST: isize = 30;
const MATCH_IPV4_SOURCE: isize = 31;
const MATCH_IPV4_DEST: isize = 32;
const MATCH_IPV6_SOURCE: isize = 33;
const MATCH_IPV6_DEST: isize = 34;
const MATCH_IP_TOS: isize = 35;
const MATCH_IP_PROTOCOL: isize = 36;
const MATCH_ETHERTYPE: isize = 37;
const MATCH_ICMP: isize = 38;
// More to come

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MatchType {
    ZtSource = MATCH_ZT_SOURCE,
    ZtDest = MATCH_ZT_DEST,
    VlanId = MATCH_VLAN_ID,
    VlanPcp = MATCH_VLAN_PCP,
    VlanDei = MATCH_VLAN_DEI,
    MacSource = MATCH_MAC_SOURCE,
    MacDest = MATCH_MAC_DEST,
    Ipv4Source = MATCH_IPV4_SOURCE,
    Ipv4Dest = MATCH_IPV4_DEST,
    Ipv6Source = MATCH_IPV6_SOURCE,
    Ipv6Dest = MATCH_IPV6_DEST,
    IpTos = MATCH_IP_TOS,
    IpProto = MATCH_IP_PROTOCOL,
    Ethertype = MATCH_ETHERTYPE,
    Icmp = MATCH_ICMP,
}

impl FromStr for MatchType {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<MatchType, Self::Err> {
        match input {
            "MATCH_ZT_SOURCE"   => Ok(MatchType::ZtSource),
            "MATCH_ZT_DEST"     => Ok(MatchType::ZtDest),
            "MATCH_VLAN_ID"     => Ok(MatchType::VlanId),
            "MATCH_VLAN_PCP"    => Ok(MatchType::VlanPcp),
            "MATCH_VLAN_DEI"    => Ok(MatchType::VlanDei),
            "MATCH_MAC_SOURCE"  => Ok(MatchType::MacSource),
            "MATCH_MAC_DEST"    => Ok(MatchType::MacDest),
            "MATCH_IPV4_SOURCE" => Ok(MatchType::Ipv4Source),
            "MATCH_IPV4_DEST"   => Ok(MatchType::Ipv4Dest),
            "MATCH_IPV6_SOURCE" => Ok(MatchType::Ipv6Source),
            "MATCH_IPV6_DEST"   => Ok(MatchType::Ipv6Dest),
            "MATCH_IP_TOS"      => Ok(MatchType::IpTos),
            "MATCH_IP_PROTOCOL" => Ok(MatchType::IpProto),
            "MATCH_ETHERTYPE"   => Ok(MatchType::Ethertype),
            "MATCH_ICMP"        => Ok(MatchType::Icmp),
            _                   => Err(ParseError::NotFound),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZtAddressMatch {
    type_id: MatchType,
    address: [u8; 5],
}
impl ZtAddressMatch {
    pub fn new(data: &BTreeMap<String, String>) -> Fallible<Self> {
        let type_id = match data.get("type") {
            Some(id) => match MatchType::from_str(id)? {
                MatchType::ZtDest   => MatchType::ZtDest,
                MatchType::ZtSource => MatchType::ZtSource,
                _                   => return Err(ParseError::NotFound.into()),
            },
            None     => return Err(ParseError::NotFound.into()),
        };

        let address = match data.get("address") {
            Some(address) => {
                let mut buf = [0u8; 5];
                buf[..].copy_from_slice(&hex::decode(address)?);
                buf
            },
            None          => return Err(ParseError::NotFound.into()),
        };

        Ok(Self {
            type_id: type_id,
            address: address,
        })
    }

    fn serialize(&self) -> Fallible<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(self.type_id as u8);
        buf.push(self.address.len() as u8);
        buf.append(&mut self.address.clone().to_vec());
        Ok(buf)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ipv4Match {
    type_id: MatchType,
    address: [u8; 4],
}
impl Ipv4Match {
    pub fn new(data: &BTreeMap<String, String>) -> Fallible<Self> {
        let type_id = match data.get("type") {
            Some(id) => match MatchType::from_str(id)? {
                MatchType::Ipv4Source => MatchType::Ipv4Source,
                MatchType::Ipv4Dest   => MatchType::Ipv4Dest,
                _                     => return Err(ParseError::NotFound.into()),
            },
            None     => return Err(ParseError::NotFound.into()),
        };

        let address = match data.get("address") {
            Some(address) => {
                std::net::Ipv4Addr::from_str(address)?.octets()
            },
            None          => return Err(ParseError::NotFound.into()),
        };

        Ok(Self {
            type_id: type_id,
            address: address,
        })
    }

    fn serialize(&self) -> Fallible<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(self.type_id as u8);
        buf.push(self.address.len() as u8);
        buf.append(&mut self.address.clone().to_vec());
        Ok(buf)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ipv6Match {
    type_id: MatchType,
    address: [u8; 16],
}
impl Ipv6Match {
    pub fn new(data: &BTreeMap<String, String>) -> Fallible<Self> {
        let type_id = match data.get("type") {
            Some(id) => match MatchType::from_str(id)? {
                MatchType::Ipv6Source => MatchType::Ipv6Source,
                MatchType::Ipv6Dest   => MatchType::Ipv6Dest,
                _                     => return Err(ParseError::NotFound.into()),
            },
            None     => return Err(ParseError::NotFound.into()),
        };

        let address = match data.get("address") {
            Some(address) => {
                std::net::Ipv6Addr::from_str(address)?.octets()
            },
            None          => return Err(ParseError::NotFound.into()),
        };

        Ok(Self {
            type_id: type_id,
            address: address,
        })
    }

    fn serialize(&self) -> Fallible<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(self.type_id as u8);
        buf.push(self.address.len() as u8);
        buf.append(&mut self.address.clone().to_vec());
        Ok(buf)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MacMatch {
    type_id: MatchType,
    address: [u8; 6],
}
impl MacMatch {
    pub fn new(data: &BTreeMap<String, String>) -> Fallible<Self> {
        let type_id = match data.get("type") {
            Some(id) => match MatchType::from_str(id)? {
                MatchType::MacSource => MatchType::MacSource,
                MatchType::MacDest   => MatchType::MacDest,
                _                    => return Err(ParseError::NotFound.into()),
            },
            None     => return Err(ParseError::NotFound.into()),
        };

        let address = match data.get("address") {
            Some(address) => {
                let mut buf = [0u8; 6];
                buf.copy_from_slice(&hex::decode(address.replace(":", ""))?);
                buf
            },
            None          => return Err(ParseError::NotFound.into()),
        };

        Ok(Self {
            type_id: type_id,
            address: address,
        })
    }

    fn serialize(&self) -> Fallible<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(self.type_id as u8);
        buf.push(self.address.len() as u8);
        buf.append(&mut self.address.clone().to_vec());
        Ok(buf)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuleMatch {
    Zt(ZtAddressMatch),
    Ipv4(Ipv4Match),
    Ipv6(Ipv6Match),
    Mac(MacMatch),
}

impl TryFrom<BTreeMap<String, String>> for RuleMatch {
    type Error = failure::Error;

    fn try_from(data: BTreeMap<String, String>) -> Result<Self, Self::Error> {
        let type_id = match data.get("type") {
            Some(id) => id,
            None     => return Err(ParseError::NotFound.into()),
        };

        match MatchType::from_str(type_id) {
            Ok(MatchType::ZtDest) | Ok(MatchType::ZtSource) => {
                Ok(Self::Zt(ZtAddressMatch::new(&data)?))
            },
            Ok(MatchType::Ipv4Dest) | Ok(MatchType::Ipv4Source) => {
                Ok(Self::Ipv4(Ipv4Match::new(&data)?))
            },
            Ok(MatchType::Ipv6Dest) | Ok(MatchType::Ipv6Source) => {
                Ok(Self::Ipv6(Ipv6Match::new(&data)?))
            },
            Ok(MatchType::MacDest) | Ok(MatchType::MacSource) => {
                Ok(Self::Mac(MacMatch::new(&data)?))
            },
            _ => Err(ParseError::NotFound.into())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Rule {
    Action(RuleAction),
    Match(RuleMatch),
}

impl TryFrom<BTreeMap<String, String>> for Rule {
    type Error = failure::Error;

    fn try_from(data: BTreeMap<String, String>) -> Result<Self, Self::Error> {
        let type_id = match data.get("type") {
            Some(id) => id,
            None     => return Err(ParseError::NotFound.into()),
        };

        if type_id.starts_with("ACTION_") {
            return Ok(Self::Action(data.try_into()?))
        } else if type_id.starts_with("MATCH_") {
            return Ok(Self::Match(data.try_into()?))
        }

        Err(ParseError::NotFound.into())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use failure::Fallible;

    #[test]
    fn test_parse_action_type_from_str() -> Fallible<()> {
        assert_eq!(ActionType::from_str("ACTION_DROP")?, ActionType::Drop);
        assert_eq!(ActionType::from_str("ACTION_ACCEPT")?, ActionType::Accept);
        assert_eq!(ActionType::from_str("ACTION_TEE")?, ActionType::Tee);
        assert_eq!(ActionType::from_str("ACTION_WATCH")?, ActionType::Watch);
        assert_eq!(ActionType::from_str("ACTION_REDIRECT")?, ActionType::Redirect);
        assert_eq!(ActionType::from_str("ACTION_BREAK")?, ActionType::Break);

        Ok(())
    }

    #[test]
    fn test_serialize_simple_action() -> Fallible<()> {
        assert_eq!(SimpleAction::new(ActionType::Drop)?.serialize()?, vec![0, 0]);
        assert_eq!(SimpleAction::new(ActionType::Accept)?.serialize()?, vec![1, 0]);
        assert_eq!(SimpleAction::new(ActionType::Break)?.serialize()?, vec![5, 0]);

        Ok(())
    }

    #[test]
    fn test_serialize_complex_action() -> Fallible<()> {
        let mut map: BTreeMap<String, String> = BTreeMap::new();

        // TEE
        map.insert(String::from("type"), String::from("ACTION_TEE"));
        map.insert(String::from("address"), String::from("aabbccddee"));
        map.insert(String::from("flags"), String::from("0"));
        map.insert(String::from("length"), String::from("20"));
        assert_eq!(ComplexAction::new(&map)?.serialize()?, vec![2, 14, 0, 0, 0, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0, 0, 0, 0, 0, 20]);

        map.clear();

        // WATCH
        map.insert(String::from("type"), String::from("ACTION_WATCH"));
        map.insert(String::from("address"), String::from("aabbccddee"));
        map.insert(String::from("flags"), String::from("0"));
        map.insert(String::from("length"), String::from("20"));
        assert_eq!(ComplexAction::new(&map)?.serialize()?, vec![3, 14, 0, 0, 0, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0, 0, 0, 0, 0, 20]);

        map.clear();

        // REDIRECT
        map.insert(String::from("type"), String::from("ACTION_REDIRECT"));
        map.insert(String::from("address"), String::from("aabbccddee"));
        map.insert(String::from("flags"), String::from("0"));
        assert_eq!(ComplexAction::new(&map)?.serialize()?, vec![4, 14, 0, 0, 0, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0, 0, 0, 0, 0, 0]);

        Ok(())
    }

    #[test]
    fn test_parse_match_type_from_str() -> Fallible<()> {
        assert_eq!(MatchType::from_str("MATCH_IPV4_SOURCE")?, MatchType::Ipv4Source);
        assert_eq!(MatchType::from_str("MATCH_ZT_DEST")?, MatchType::ZtDest);
        assert_eq!(MatchType::from_str("MATCH_ICMP")?, MatchType::Icmp);
        assert_eq!(MatchType::from_str("MATCH_ETHERTYPE")?, MatchType::Ethertype);
        assert_eq!(MatchType::from_str("MATCH_MAC_DEST")?, MatchType::MacDest);
        assert_eq!(MatchType::from_str("MATCH_IP_TOS")?, MatchType::IpTos);

        Ok(())
    }

    #[test]
    fn test_serialize_zt_address_match() -> Fallible<()> {
        let mut map: BTreeMap<String, String> = BTreeMap::new();

        // ZT_SOURCE
        map.insert(String::from("type"), String::from("MATCH_ZT_SOURCE"));
        map.insert(String::from("address"), String::from("aabbccddee"));
        assert_eq!(ZtAddressMatch::new(&map)?.serialize()?, vec![MATCH_ZT_SOURCE as u8, 5, 0xaa, 0xbb, 0xcc, 0xdd, 0xee]);

        map.clear();

        // ZT_DEST
        map.insert(String::from("type"), String::from("MATCH_ZT_DEST"));
        map.insert(String::from("address"), String::from("aabbccddee"));
        assert_eq!(ZtAddressMatch::new(&map)?.serialize()?, vec![MATCH_ZT_DEST as u8, 5, 0xaa, 0xbb, 0xcc, 0xdd, 0xee]);

        Ok(())
    }

    #[test]
    fn test_serialize_ip_address_match() -> Fallible<()> {
        let mut map: BTreeMap<String, String> = BTreeMap::new();

        // IPV4_SOURCE
        map.insert(String::from("type"), String::from("MATCH_IPV4_SOURCE"));
        map.insert(String::from("address"), String::from("100.100.0.50"));
        assert_eq!(Ipv4Match::new(&map)?.serialize()?, vec![MATCH_IPV4_SOURCE as u8, 4, 100, 100, 0, 50]);

        map.clear();

        // IPV4_DEST
        map.insert(String::from("type"), String::from("MATCH_IPV4_DEST"));
        map.insert(String::from("address"), String::from("100.100.0.50"));
        assert_eq!(Ipv4Match::new(&map)?.serialize()?, vec![MATCH_IPV4_DEST as u8, 4, 100, 100, 0, 50]);

        map.clear();

        // IPV6_SOURCE
        map.insert(String::from("type"), String::from("MATCH_IPV6_SOURCE"));
        map.insert(String::from("address"), String::from("2001:0db8:85a3:0000:0000:8a2e:0370:7334"));
        assert_eq!(Ipv6Match::new(&map)?.serialize()?, vec![MATCH_IPV6_SOURCE as u8, 16, 32, 1, 13, 184, 133, 163, 0, 0, 0, 0, 138, 46, 3, 112, 115, 52]);

        map.clear();

        // IPV6_DEST
        map.insert(String::from("type"), String::from("MATCH_IPV6_DEST"));
        map.insert(String::from("address"), String::from("2001:0db8:85a3:0000:0000:8a2e:0370:7334"));
        assert_eq!(Ipv6Match::new(&map)?.serialize()?, vec![MATCH_IPV6_DEST as u8, 16, 32, 1, 13, 184, 133, 163, 0, 0, 0, 0, 138, 46, 3, 112, 115, 52]);

        Ok(())
    }

    #[test]
    fn test_serialize_mac_address_match() -> Fallible<()> {
        let mut map: BTreeMap<String, String> = BTreeMap::new();

        // MAC_SOURCE
        map.insert(String::from("type"), String::from("MATCH_MAC_SOURCE"));
        map.insert(String::from("address"), String::from("ff:ff:ff:ff:ff:ff"));
        assert_eq!(MacMatch::new(&map)?.serialize()?, vec![MATCH_MAC_SOURCE as u8, 6, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);

        map.clear();

        // MAC_DEST
        map.insert(String::from("type"), String::from("MATCH_MAC_DEST"));
        map.insert(String::from("address"), String::from("ff:ff:ff:ff:ff:ff"));
        assert_eq!(MacMatch::new(&map)?.serialize()?, vec![MATCH_MAC_DEST as u8, 6, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);

        Ok(())
    }

    #[test]
    fn test_map_into_rule() -> Fallible<()> {
        let mut map: BTreeMap<String, String> = BTreeMap::new();

        // MATCH_MAC_SOURCE
        map.insert(String::from("type"), String::from("MATCH_MAC_SOURCE"));
        map.insert(String::from("address"), String::from("ff:ff:ff:ff:ff:ff"));
        let rule: Rule = map.clone().try_into()?;
        assert_eq!(rule, Rule::Match(RuleMatch::Mac(MacMatch{ type_id: MatchType::MacSource, address: [0xff, 0xff, 0xff, 0xff, 0xff, 0xff] })));

        map.clear();

        // ACTION_ACCEPT
        map.insert(String::from("type"), String::from("ACTION_ACCEPT"));
        let rule: Rule = map.clone().try_into()?;
        assert_eq!(rule, Rule::Action(RuleAction::Accept(SimpleAction(ActionType::Accept))));

        map.clear();

        // MATCH_IPV6_DEST
        map.insert(String::from("type"), String::from("MATCH_IPV6_DEST"));
        map.insert(String::from("address"), String::from("2001:0db8:85a3:0000:0000:8a2e:0370:7334"));
        let rule: Rule = map.clone().try_into()?;
        assert_eq!(rule, Rule::Match(
                RuleMatch::Ipv6(
                    Ipv6Match {
                        type_id: MatchType::Ipv6Dest,
                        address: [32, 1, 13, 184, 133, 163, 0, 0, 0, 0, 138, 46, 3, 112, 115, 52]
                    }
                )
            )
        );

        map.clear();

        // ACTION_WATCH
        map.insert(String::from("type"), String::from("ACTION_WATCH"));
        map.insert(String::from("address"), String::from("aabbccddee"));
        map.insert(String::from("flags"), String::from("0"));
        map.insert(String::from("length"), String::from("20"));
        let rule: Rule = map.clone().try_into()?;
        assert_eq!(rule, Rule::Action(
                RuleAction::Watch(
                    ComplexAction {
                        type_id: ActionType::Watch,
                        address: 0xaabbccddee,
                        flags: 0,
                        length: 20,
                    }
                )
            )
        );

        Ok(())
    }
}
