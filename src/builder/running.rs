use job;
use uuid::Uuid;

pub struct Job
{
    /// The UUID of the node that is requesting the job.
    pub origin: Uuid,
    pub job: job::Job,
}

