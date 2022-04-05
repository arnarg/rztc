use zt::core::Node;
use zt::identity::Identity;

fn main() {
    let identity_str = std::fs::read_to_string("/tmp/rztc/identity.secret").unwrap();
    let identity = Identity::try_from(identity_str.as_str()).unwrap();
    let node = Node::new(Box::new(identity)).unwrap();
    node.process_background_tasks().unwrap();
}
