use {Network, Edge};
use std::borrow::Cow;

use uuid::Uuid;

use dot;
use std;

impl<'a> dot::Labeller<'a, Uuid, Edge> for &'a Network {
    fn graph_id(&'a self) -> dot::Id<'a> { dot::Id::new("network").unwrap() }

    fn node_id(&'a self, n: &Uuid) -> dot::Id<'a> {
        dot::Id::new(format!("{}", n.simple())).unwrap()
    }
}

impl<'a> dot::GraphWalk<'a, Uuid, Edge> for &'a Network {
    fn nodes(&self) -> dot::Nodes<'a, Uuid> {
        Cow::Owned(self.nodes.values().map(|n| n.uuid.clone()).collect())
    }

    fn edges(&'a self) -> dot::Edges<'a, Edge> {
        Cow::Borrowed(&self.edges)
    }

    fn source(&self, e: &Edge) -> Uuid { e.a }

    fn target(&self, e: &Edge) -> Uuid { e.b }
}

pub fn render_to<W: std::io::Write>(network: &Network, output: &mut W) {
    dot::render(&network, output).unwrap()
}

