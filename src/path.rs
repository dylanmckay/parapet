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
    pub fn new(hops: VecDeque<Uuid>) -> Self {
        Path { hops: hops }
    }
}

