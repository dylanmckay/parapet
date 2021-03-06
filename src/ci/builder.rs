use ci;

use uuid::Uuid;

use std::collections::{HashMap, VecDeque};
use std::sync::mpsc;

pub struct Builder
{
    tx: mpsc::Sender<ci::build::WorkOutput>,
    rx: mpsc::Receiver<ci::build::WorkOutput>,

    running_work: HashMap<Uuid, RunningWork>,
    completed_work: VecDeque<CompletedWork>,
}

pub struct RunningWork
{
    /// The UUID of the node that is requesting the work.
    pub origin: Uuid,
    pub work: ci::build::Work,
}

pub struct CompletedWork
{
    /// The UUID of the node that is requesting the work.
    pub origin: Uuid,
    pub output: ci::build::WorkOutput,
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

    pub fn build(&mut self, origin: Uuid, work: ci::build::Work) {
        let tx = self.tx.clone();

        let pending_work = RunningWork { origin: origin, work: work.clone() };

        self.running_work.insert(work.uuid, pending_work);

        let ci = ci::sandbox::Basic;
        ci::build::work(work, Box::new(ci), tx);
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

    /// Decides whether or not we are ready to do more work.
    pub fn should_pickup_work(&self) -> bool {
        true
    }
}

