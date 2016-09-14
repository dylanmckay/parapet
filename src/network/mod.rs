pub use self::path::Path;

pub mod path;

use Connection;
use std::collections::{HashMap, VecDeque};

use uuid::Uuid;
use graphsearch;

pub struct Weight(f32);

#[derive(Debug)]
pub struct Network
{
    pub nodes: HashMap<Uuid, Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug)]
pub struct Node
{
    pub uuid: Uuid,
    pub connection: Option<Connection>,
}

/// An edge betweeb two nodes.
///
/// Node `a` is always the smaller UUID, and node `b` is
/// always the bigger one.
#[derive(Clone, Debug, PartialEq)]
pub struct Edge
{
    pub a: Uuid,
    pub b: Uuid,
}

impl Edge
{
    pub fn new(node1: Uuid, node2: Uuid) -> Edge {
        let mut sorted = [node1, node2];
        sorted.sort();

        Edge {
            a: sorted[0],
            b: sorted[1],
        }
    }

    pub fn connected_to(&self, uuid: &Uuid) -> bool {
        [&self.a, &self.b].iter().any(|adjacent| &uuid == adjacent)
    }

    pub fn other_node(&self, uuid: &Uuid) -> Option<&Uuid> {
        vec![&self.a, &self.b].into_iter().filter(|a| a == &uuid).next()
    }
}

impl Network
{
    pub fn empty() -> Self {
        Network {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn new(your_uuid: Uuid) -> Self {
        let mut network = Network::empty();
        network.insert(Node {
            uuid: your_uuid,
            connection: None,
        });

        network
    }

    pub fn insert(&mut self, node: Node) {
        self.nodes.insert(node.uuid.clone(), node);
    }

    pub fn get(&self, uuid: &Uuid) -> Option<&Node> {
        self.nodes.get(uuid)
    }

    pub fn get_mut(&mut self, uuid: &Uuid) -> Option<&mut Node> {
        self.nodes.get_mut(uuid)
    }

    pub fn lookup_token_mut(&mut self, token: ::mio::Token) -> Option<&mut Node> {
        self.nodes.values_mut().find(|node| node.connection.as_ref().map_or(false, |c| c.token == token))
    }

    pub fn nodes<'a>(&'a self) -> impl Iterator<Item=&'a Node> {
        self.nodes.values()
    }

    pub fn connect(&mut self, a: Uuid, b: Uuid) {
        let edge = Edge::new(a, b);

        if self.edges.iter().find(|&e| e == &edge).is_none() {
            self.edges.push(edge);
        }
    }

    pub fn siblings(&self, node: &Uuid) -> Vec<&Uuid> {
        self.edges.iter().filter_map(|edge| {
            if edge.connected_to(node) {
                Some(edge.other_node(node).unwrap())
            } else {
                None
            }
        }).collect()
    }

    pub fn set_connection(&mut self, uuid: &Uuid, connection: Connection) {
        self.nodes.get_mut(uuid).unwrap().connection = Some(connection);
    }

    pub fn build_graph(&self) -> graphsearch::Graph<Uuid> {
        let raw_graph: Vec<_> = self.nodes.values().map(|node| {
            graphsearch::Node {
                content: node.uuid.clone(),
                adjacent: self.siblings(&node.uuid).into_iter().map(|&sibling| {
                    let sibling_idx = self.nodes.values().position(|n| n.uuid == sibling).unwrap();

                    graphsearch::Vertex { cost: 1, node: sibling_idx }
                }).collect(),
            }
        }).collect();

        graphsearch::Graph::new(raw_graph)
    }

    pub fn shortest_path(&self, from: Uuid, to: Uuid) -> VecDeque<Uuid> {
        let graph = self.build_graph();

        let from_idx = self.nodes.values().position(|node| node.uuid == from).unwrap();

        graph.search(from_idx, to).unwrap().into_iter().map(|idx| {
            self.nodes.values().nth(idx).unwrap().uuid
        }).collect()
    }
}

