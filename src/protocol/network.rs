use network;
use uuid::Uuid;

define_composite_type!(Network {
    nodes: Vec<Node>,
    edges: Vec<Edge>
});

define_composite_type!(Node {
    // The UUID of the node.
    uuid: Uuid,
    // The status of the node, if it is remote.
    status: Option<NodeStatus>
});

// Defines the status of a remote node.
define_composite_type!(NodeStatus {
    work_available: bool
});

define_composite_type!(Edge {
    a: Uuid,
    b: Uuid
});

impl Network
{
    pub fn from_network(network: &network::Network) -> Self {
        Network {
            nodes: network.nodes.iter().map(|(_,n)| Node::from_node(n)).collect(),
            edges: network.edges.iter().map(|e| Edge::from_edge(e)).collect(),
        }
    }
}

impl Node
{
    pub fn from_node(node: &network::Node) -> Self {
        Node {
            uuid: node.uuid.clone(),
            status: match node.status {
                network::Status::Local => None,
                network::Status::Remote(ref status) => Some(NodeStatus::from_remote_status(status)),
            },
        }
    }
}

impl Edge
{
    fn from_edge(edge: &network::Edge) -> Self {
        Edge {
            a: edge.a.clone(),
            b: edge.b.clone(),
        }
    }
}

impl NodeStatus
{
    fn from_remote_status(status: &network::remote::Status) -> Self {
        NodeStatus {
            work_available: if let network::remote::status::Work::Available { .. } = status.work { true } else { false },
        }
    }
}

impl Into<network::Network> for Network
{
    fn into(self) -> network::Network {
        network::Network {
            nodes: self.nodes.into_iter().map(|n| (n.uuid.clone(), n.into())).collect(),
            edges: self.edges.into_iter().map(|n| n.into()).collect(),
        }
    }
}

impl Into<network::Node> for Node
{
    fn into(self) -> network::Node {
        network::Node {
            uuid: self.uuid,
            connection: None,
            status: match self.status {
                Some(status) => network::Status::Remote(status.into()),
                None => network::Status::Local,
            }
        }
    }
}

impl Into<network::Edge> for Edge
{
    fn into(self) -> network::Edge {
        network::Edge {
            a: self.a,
            b: self.b,
        }
    }
}

impl Into<network::remote::Status> for NodeStatus
{
    fn into(self) -> network::remote::Status {
        let work = if self.work_available {
            network::remote::status::Work::Available { have_asked_for_work: false }
        } else {
            network::remote::status::Work::Unavailable
        };

        network::remote::Status { work: work }
    }
}

