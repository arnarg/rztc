extern crate hex;

mod phy;
mod config;

use std::time::{SystemTime, UNIX_EPOCH};
use zt::core::Node;
use phy::Phy;
use config::FileConfig;

fn main() {
    let file_config = FileConfig::new("/tmp/rztc/identity.secret");
    let node = Node::new(Box::new(file_config)).unwrap();
    let mut phy = Phy::new().unwrap();

    let mut online = node.is_online();
    let mut next: i64 = 0;
    loop {
        // Poll sockets for incoming packets
        if let Err(error) = phy.poll(&node) {
            println!("poll failed: {}", error);
        }

        // Get current time in milliseconds since epoch
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("unable to get time in millis");
        let now: i64 = now.as_millis().try_into().unwrap();
        // Process background tasks in node
        if next < now {
            match node.process_background_tasks(&phy) {
                Ok(next_deadline) => next = next_deadline,
                Err(err) => println!("process_background_tasks failed: {}", err),
            }
        }

        let node_online = node.is_online();
        //println!("{}", node_online);
        if online != node_online {
            println!("node status changed: {}", if node_online { "online" } else { "offline" });
            online = node_online;
        }
    }
}
