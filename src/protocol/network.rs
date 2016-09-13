use uuid::Uuid;

define_composite_type!(Network {
    nodes: Vec<Node>,
    edges: Vec<Edge>
});

define_composite_type!(Node {
    uuid: Uuid
});

define_composite_type!(Edge {
    a: Uuid,
    b: Uuid
});

impl Network
{
    pub fn from_network(network: &::Network) -> Self {
        Network {
            nodes: network.nodes.iter().map(|(_,n)| Node::from_node(n)).collect(),
            edges: network.edges.iter().map(|e| Edge::from_edge(e)).collect(),
        }
    }
}

impl Node
{
    pub fn from_node(node: &::Node) -> Self {
        Node {
            uuid: node.uuid.clone(),
        }
    }
}

impl Edge
{
    fn from_edge(edge: &::Edge) -> Self {
        Edge {
            a: edge.a.clone(),
            b: edge.b.clone(),
        }
    }
}

impl Into<::Network> for Network
{
    fn into(self) -> ::Network {
        ::Network {
            nodes: self.nodes.into_iter().map(|n| (n.uuid.clone(), n.into())).collect(),
            edges: self.edges.into_iter().map(|n| n.into()).collect(),
        }
    }
}

impl Into<::Node> for Node
{
    fn into(self) -> ::Node {
        ::Node {
            uuid: self.uuid,
            connection: None,
        }
    }
}

impl Into<::Edge> for Edge
{
    fn into(self) -> ::Edge {
        ::Edge {
            a: self.a,
            b: self.b,
        }
    }
}

