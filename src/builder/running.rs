use workspace;
use uuid::Uuid;

pub struct Work
{
    /// The UUID of the node that is requesting the work.
    pub origin: Uuid,
    pub work: workspace::build::Work,
}

