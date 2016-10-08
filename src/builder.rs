use job;
use uuid::Uuid;

use std::collections::{HashMap, VecDeque};
use std::sync::mpsc;

pub struct Builder
{
    tx: mpsc::Sender<job::run::JobOutput>,
    rx: mpsc::Receiver<job::run::JobOutput>,

    pending_jobs: HashMap<Uuid, PendingJob>,
    completed_jobs: VecDeque<CompletedJob>,
}

pub struct PendingJob
{
    /// The UUID of the node that is requesting the job.
    pub origin: Uuid,
    pub job: job::Job,
}

pub struct CompletedJob
{
    /// The UUID of the node that is requesting the job.
    pub origin: Uuid,
    pub output: job::run::JobOutput,
}

impl Builder
{
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        Builder {
            tx: tx,
            rx: rx,
            pending_jobs: HashMap::new(),
            completed_jobs: VecDeque::new(),
        }
    }

    pub fn build(&mut self, origin: Uuid, job: job::Job) {
        let tx = self.tx.clone();

        let pending_job = PendingJob { origin: origin, job: job.clone() };

        self.pending_jobs.insert(job.uuid, pending_job);
        job::run::job(job, tx);
    }

    pub fn tick(&mut self) {
        loop {
            match self.rx.try_recv() {
                Ok(output) => {
                    let pending_job = self.pending_jobs.remove(&output.job.uuid).unwrap();

                    println!("job complete: {:?}", output);

                    self.completed_jobs.push_back(CompletedJob {
                        origin: pending_job.origin,
                        output: output,
                    });
                },
                Err(..) => break,
            }
        }
    }

    pub fn completed_jobs(&mut self) -> ::std::collections::vec_deque::Drain<CompletedJob> {
        self.completed_jobs.drain(..)
    }
}

