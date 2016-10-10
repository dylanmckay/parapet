use std::collections::VecDeque;
use uuid::Uuid;

use std::fmt;

/// A path through the network.
#[derive(Clone, PartialEq)]
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

    pub fn sender(&self) -> &Uuid {
        self.head().expect("packet has no sender")
    }

    pub fn head(&self) -> Option<&Uuid> {
        self.hops.back()
    }

    pub fn tail(&self) -> Option<&Uuid> {
        self.hops.front()
    }

    pub fn next_hop(&self, from: &Uuid) -> Option<Uuid> {
        // Create an iterator from the first node to the end.
        self.hops.iter().rev().skip_while(|node| node != &from).skip(1).next().cloned()
    }

    pub fn ends_at(&self, node: &Uuid) -> bool {
        // If there are no hops, the path can end at all nodes.
        self.hops.front().map(|uuid| uuid == node).unwrap_or(true)
    }
}

impl fmt::Debug for Path
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let hops: Vec<_> = self.hops.iter().rev().map(|uuid| format!("{}", uuid)).collect();
        hops.join(" -> ").fmt(fmt)
    }
}

#[cfg(test)]
mod test
{
    pub use std::collections::VecDeque;
    pub use uuid::Uuid;
    pub use network::Path;

    describe! path {
        describe! next_hop {
            before_each {
                let node1 = Uuid::parse_str("123e4567-e89b-12d3-a456-426655440000").unwrap();
                let node2 = Uuid::parse_str("2c6bb858-72ea-4db6-a9ab-6ec0ca1f18ab").unwrap();
                let node3 = Uuid::parse_str("3042be80-4d2b-4cce-b30c-3142e3035720").unwrap();

                let mut path_queue = VecDeque::new();
                path_queue.push_front(node1.clone());
                path_queue.push_front(node2.clone());
                path_queue.push_front(node3.clone());

                let path = Path::new(path_queue);
            }

            it "recognizes the second hop" {
                assert_eq!(path.next_hop(&node1).unwrap(), node2);
            }

            it "recognizes the third hop" {
                assert_eq!(path.next_hop(&node2).unwrap(), node3);
            }

            it "returns None when the given UUID is not in the path" {
                let invalid_uuid = Uuid::parse_str("883c3194-6750-4bbb-8f38-b2b05fc8a40e").unwrap();

                assert_eq!(path.next_hop(&invalid_uuid), None);
            }
        }
    }
}

