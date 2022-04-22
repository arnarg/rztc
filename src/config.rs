use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_secondary_port")]
    pub secondary_port: u16,
    pub identity_path: String,
    networks: Vec<Network>,
}

fn default_port() -> u16 { 9994 }
fn default_secondary_port() -> u16 { 29995 }

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Network {
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
    rules: Vec<Rule>,
}

fn default_revision() -> u64 { 0 }
fn default_public() -> bool { false }
fn default_broadcast() -> bool { true }
fn default_multicast() -> u64 { 32 }
fn default_mtu() -> u16 { 2800 }

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Route {
    destination: String,
    via: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct DNS {
    search_domain: String,
    server_address: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Member {
    address: String,
    ip: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
