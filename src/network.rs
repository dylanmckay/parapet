use Node;
use std::collections::HashMap;

use uuid::Uuid;

pub struct Network
{
    pub nodes: HashMap<Uuid, Node>,
}

impl Network
{
    pub fn new() -> Self {
        Network {
            nodes: HashMap::new(),
        }
    }

    pub fn insert(&mut self, node: Node) {
        self.nodes.insert(node.uuid.clone(), node);
    }

    pub fn lookup_token_mut(&mut self, token: ::mio::Token) -> Option<&mut Node> {
        self.nodes.values_mut().find(|node| node.connection.as_ref().map_or(false, |c| c.token == token))
    }
}

