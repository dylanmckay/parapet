use network::remote;

/// The status of a node.
#[derive(Clone, Debug)]
pub enum Status
{
    /// The node is remote.
    Remote(remote::Status),
    /// The node is local.
    Local,
}

