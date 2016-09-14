use std::collections::VecDeque;
use uuid::Uuid;

/// A path through the network.
#[derive(Clone, Debug)]
pub struct Path
{
    pub hops: VecDeque<Uuid>,
}

implement_composite_type!(Path { hops });

impl Path
{
    pub fn empty() -> Self {
        Path { hops: VecDeque::new() }
    }

    pub fn new(hops: VecDeque<Uuid>) -> Self {
        Path { hops: hops }
    }

    pub fn from_to(from: Uuid, to: Uuid) -> Self {
        Self::empty().bounce(from).bounce(to)
    }

    pub fn bounce(mut self, node: Uuid) -> Self {
        self.hops.push_front(node);
        self
    }
}

