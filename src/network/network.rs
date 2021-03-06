use network::{Node, Connection, Path, Status};
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

pub struct Entry<'a>
{
    uuid: Uuid,
    network: &'a mut Network,
}

impl<'a> Entry<'a>
{
    pub fn remove(self) -> Node {
        self.network.edges = self.network.edges.iter().cloned().filter(|edge| {
            !edge.connected_to(&self.uuid)
        }).collect();

        self.network.nodes.remove(&self.uuid).unwrap()
    }

    pub fn get(&self) -> &Node {
        self.network.nodes.get(&self.uuid).unwrap()
    }

    pub fn get_mut(&mut self) -> &mut Node {
        self.network.nodes.get_mut(&self.uuid).unwrap()
    }
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
        vec![&self.a, &self.b].into_iter().filter(|a| a != &uuid).next()
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
            status: Status::Local,
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

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn lookup_token_mut(&mut self, token: ::mio::Token) -> Option<&mut Node> {
        self.nodes.values_mut().find(|node| node.connection.as_ref().map_or(false, |c| c.token == token))
    }

    pub fn entry_by_token(&mut self, token: ::mio::Token) -> Option<Entry> {
        let uuid = self.nodes.values().find(|node| node.connection.as_ref().map_or(false, |c| c.token == token)).map(|n| n.uuid.clone());

        if let Some(uuid) = uuid {
            Some(Entry { uuid: uuid, network: self })
        } else {
            None
        }
    }

    pub fn nodes<'a>(&'a self) -> impl Iterator<Item=&'a Node> {
        self.nodes.values()
    }

    pub fn connect(&mut self, a: &Uuid, b: &Uuid) {
        let edge = Edge::new(a.clone(), b.clone());

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

    pub fn route(&self, from: &Uuid, to: &Uuid) -> Path {
        Path::new(self.shortest_path(from, to))
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

    pub fn shortest_path(&self, from: &Uuid, to: &Uuid) -> VecDeque<Uuid> {
        println!("route from {} to {}", from, to);
        let graph = self.build_graph();

        let from_idx = self.nodes.values().position(|node| &node.uuid == from).unwrap();

        graph.search(from_idx, to.clone()).expect("no route exists").into_iter().map(|idx| {
            self.nodes.values().nth(idx).unwrap().uuid
        }).rev().collect()
    }
}

