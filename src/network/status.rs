use network::remote;

/// The status of a node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status
{
    /// The node is remote.
    Remote(remote::Status),
    /// The node is local.
    Local,
}

impl Status
{
    pub fn expect_remote_mut(&mut self) -> &mut remote::Status {
        match *self {
            Status::Remote(ref mut s) => s,
            _ => panic!("expected a remote node but found a local one"),
        }
    }
}

