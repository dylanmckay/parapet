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

impl From<::Network> for Network
{
    fn from(network: ::Network) -> Self {
        Network {
            nodes: network.nodes.into_iter().map(|(_,n)| Node::from(n)).collect(),
            edges: network.edges.into_iter().map(|e| Edge::from(e)).collect(),
        }
    }
}

impl From<::Node> for Node
{
    fn from(node: ::Node) -> Self {
        Node {
            uuid: node.uuid,
        }
    }
}

impl From<::Edge> for Edge
{
    fn from(edge: ::Edge) -> Self {
        Edge {
            a: edge.a,
            b: edge.b,
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

