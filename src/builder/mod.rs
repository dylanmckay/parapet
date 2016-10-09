use {job, workspace};

use uuid::Uuid;

use std::collections::{HashMap, VecDeque};
use std::sync::mpsc;

pub mod running;

pub struct Builder
{
    tx: mpsc::Sender<job::run::WorkOutput>,
    rx: mpsc::Receiver<job::run::WorkOutput>,

    running_work: HashMap<Uuid, running::Work>,
    completed_work: VecDeque<CompletedWork>,
}

pub struct CompletedWork
{
    /// The UUID of the node that is requesting the work.
    pub origin: Uuid,
    pub output: job::run::WorkOutput,
}

impl Builder
{
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        Builder {
            tx: tx,
            rx: rx,
            running_work: HashMap::new(),
            completed_work: VecDeque::new(),
        }
    }

    pub fn build(&mut self, origin: Uuid, work: job::Work) {
        let tx = self.tx.clone();

        let pending_work = running::Work { origin: origin, work: work.clone() };

        self.running_work.insert(work.uuid, pending_work);

        let workspace = workspace::basic::Basic;
        job::run::work(work, Box::new(workspace), tx);
    }

    pub fn tick(&mut self) {
        loop {
            match self.rx.try_recv() {
                Ok(output) => {
                    let pending_work = self.running_work.remove(&output.work.uuid).unwrap();

                    println!("work complete: {:?}", output);

                    self.completed_work.push_back(CompletedWork {
                        origin: pending_work.origin,
                        output: output,
                    });
                },
                Err(..) => break,
            }
        }
    }

    pub fn completed_work(&mut self) -> ::std::collections::vec_deque::Drain<CompletedWork> {
        self.completed_work.drain(..)
    }
}

