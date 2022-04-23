extern crate hex;
extern crate serde;
extern crate serde_yaml;
extern crate clap;

mod phy;
mod identity;
mod config;

use std::time::{SystemTime, UNIX_EPOCH};
use zt::core::Node;
use zt::controller::Controller;
use phy::Phy;
use identity::IdentityState;
use failure::Fallible;
use clap::Parser;

pub struct NodeRunner {
    node: Node,
    phy: Phy,
}

impl NodeRunner {
    pub fn new(node: Node, phy: Phy) -> Self {
        Self {
            node: node,
            phy: phy,
        }
    }

    pub fn run(&mut self) -> Fallible<()> {
        let mut online = self.node.is_online();
        let mut next: i64 = 0;

        loop {
            // Poll sockets for incoming packets
            if let Err(error) = self.phy.poll(&self.node) {
                println!("poll failed: {}", error);
            }

            // Get current time in milliseconds since epoch
            let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("unable to get time in millis");
            let now: i64 = now.as_millis().try_into().unwrap();
            // Process background tasks in node
            if next < now {
                match self.node.process_background_tasks(&self.phy) {
                    Ok(next_deadline) => next = next_deadline,
                    Err(err) => println!("process_background_tasks failed: {}", err),
                }
            }

            let node_online = self.node.is_online();
            if online != node_online {
                println!("node status changed: {}", if node_online { "online" } else { "offline" });
                online = node_online;
            }
        }
    }
}

fn init_controller(node: &mut Node, conf: &config::Config) -> Fallible<()> {
    let mut controller = Controller::new();

    for n in &conf.networks {
        controller.add_network(n.clone().try_into()?);
    }

    node.register_controller(Box::new(controller))?;

    Ok(())
}

fn run(conf: config::Config) -> Fallible<()> {
    let identity_state = IdentityState::new(conf.identity_path.as_str());

    let mut node = Node::new(Box::new(identity_state))?;
    init_controller(&mut node, &conf)?;

    println!("libzerotierone v{}", node.version());

    let phy = Phy::new(conf.port, conf.secondary_port).unwrap();
    let mut runner = NodeRunner::new(node, phy);

    runner.run()?;

    Ok(())
}

/// ZeroTier network controller
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to config file
    #[clap(short, long)]
    config: String,
}

fn main() -> Fallible<()> {
    let args = Args::parse();

    let conf: config::Config = serde_yaml::from_str(
		&*std::fs::read_to_string(args.config.as_str())
			.expect(&format!("Could not open file {}", args.config)))
		.expect("Could not parse the configuration yaml file");

    run(conf)?;

    Ok(())
}
