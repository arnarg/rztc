extern crate hex;

mod phy;
mod config;

use std::time::{SystemTime, UNIX_EPOCH};
use zt::core::Node;
use zt::controller::Controller;
use phy::Phy;
use config::FileConfig;
use failure::Fallible;

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
            //println!("{}", node_online);
            if online != node_online {
                println!("node status changed: {}", if node_online { "online" } else { "offline" });
                online = node_online;
            }
        }
    }
}

fn main() {
    let file_config = FileConfig::new("/tmp/rztc/identity.secret");
    let mut node = Node::new(Box::new(file_config)).unwrap();
    node.register_controller(Box::new(Controller::new())).unwrap();
    println!("libzerotierone v{}", node.version());
    let phy = Phy::new(9993).unwrap();
    let mut runner = NodeRunner::new(node, phy);

    match runner.run() {
        Err(error) => println!("runner exited with error: {}", error),
        Ok(_) => (),
    };
}
