use job;

use std::collections::{HashMap, VecDeque};

use uuid::Uuid;

pub struct Dispatcher
{
    pub pending_jobs: VecDeque<job::Job>,
    pub running_jobs: VecDeque<RunningJob>,
    pub completed_jobs: VecDeque<CompletedJob>,
}

pub struct RunningJob
{
    pub job: job::Job,
    pub pending_tasks: VecDeque<job::Task>,
    pub running_work: HashMap<Uuid, RunningWork>,
    pub completed_tasks: VecDeque<job::run::TaskResult>,
}

pub struct CompletedJob
{
    pub job: job::Job,
    pub task_results: VecDeque<job::run::TaskResult>,
}

pub struct RunningWork
{
    pub uuid: Uuid,
    pub running_tasks: HashMap<Uuid, job::Task>,
    pub completed_tasks: VecDeque<job::run::TaskResult>,
}

pub struct Work
{
    pub uuid: Uuid,
    pub tasks: VecDeque<job::Task>,
}

pub struct CompletedWork
{
    pub uuid: Uuid,
    pub task_results: VecDeque<job::run::TaskResult>,
}

impl Dispatcher
{
    pub fn new() -> Self {
        Dispatcher {
            pending_jobs: VecDeque::new(),
            running_jobs: VecDeque::new(),
            completed_jobs: VecDeque::new(),
        }
    }

    pub fn enqueue(&mut self, job: job::Job) {
        self.pending_jobs.push_back(job);
    }

    /// Poll the dispatcher for work.
    pub fn poll(&mut self) -> Option<Work> {
        if self.running_jobs.is_empty() {
            if let Some(pending_job) = self.pending_jobs.pop_front() {
                // We may need to move the next job onto the queue.
                self.running_jobs.push_back(RunningJob {
                    pending_tasks: pending_job.tasks.iter().cloned().collect(),
                    running_work: HashMap::new(),
                    completed_tasks: VecDeque::new(),
                    job: pending_job,
                });
            }
        }

        if let Some(running_job) = self.running_jobs.front_mut() {
            let tasks = if let Some(pending_task) = running_job.pending_tasks.pop_front() {
                vec![pending_task]
            } else {
                vec![]
            };

            let work = Work {
                uuid: Uuid::new_v4(),
                tasks: tasks.into_iter().collect(),
            };

            let running_work = RunningWork {
                uuid: work.uuid.clone(),
                running_tasks: work.tasks.iter().map(|t| (t.uuid.clone(), t.clone())).collect(),
                completed_tasks: VecDeque::new(),
            };

            running_job.running_work.insert(running_work.uuid.clone(), running_work);

            Some(work)
        } else {
            None
        }
    }

    pub fn complete(&mut self, work: CompletedWork) {
        {
            let job_uuid = self.find_job_uuid_containing_work_uuid(&work.uuid).unwrap();
            let mut running_job = self.running_jobs.iter_mut().find(|job| job.job.uuid == job_uuid).unwrap();
            let mut running_work = running_job.running_work.remove(&work.uuid).unwrap();

            running_work.completed_tasks.extend(work.task_results);
            running_job.completed_tasks.extend(running_work.completed_tasks);
        }

        self.move_finished_jobs();
    }

    fn move_finished_jobs(&mut self) {
        let finished_indices: Vec<_> = self.running_jobs.iter().
            enumerate().
            filter(|&(_, job)| job.is_complete()).
            map(|a| a.0).
            collect();

        let mut completed_jobs = Vec::new();
        for idx in finished_indices {
            let running_job = self.running_jobs.remove(idx).unwrap();

            completed_jobs.push(CompletedJob {
                job: running_job.job,
                task_results: running_job.completed_tasks,
            });
        }
    }

    fn find_job_uuid_containing_work_uuid(&self, work_uuid: &Uuid) -> Option<Uuid> {
        self.running_jobs.iter().find(|job| job.running_work.contains_key(work_uuid)).map(|job| job.job.uuid)
    }
}

impl RunningJob
{
    pub fn is_complete(&self) -> bool {
        self.pending_tasks.is_empty() && self.running_work.is_empty()
    }
}

