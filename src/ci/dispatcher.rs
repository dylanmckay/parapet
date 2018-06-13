use ci::{self, Job, Task};

use std::collections::{HashMap, VecDeque};

use uuid::Uuid;

pub struct Dispatcher
{
    pending_jobs: VecDeque<Job>,
    running_jobs: VecDeque<RunningJob>,
    completed_jobs: VecDeque<CompletedJob>,
}

struct RunningJob
{
    job: Job,
    pending_tasks: VecDeque<Task>,
    running_work: HashMap<Uuid, RunningWork>,
    completed_tasks: Vec<ci::build::TaskResult>,
}

pub struct CompletedJob
{
    pub job: Job,
    pub task_results: Vec<ci::build::TaskResult>,
}

struct RunningWork
{
    uuid: Uuid,
    running_tasks: HashMap<Uuid, Task>,
    completed_tasks: Vec<ci::build::TaskResult>,
}

pub struct CompletedWork
{
    pub uuid: Uuid,
    pub task_results: Vec<ci::build::TaskResult>,
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

    /// Adds a new job to the queue.
    pub fn enqueue(&mut self, job: Job) {
        self.pending_jobs.push_back(job);
    }

    /// Poll the dispatcher for work.
    pub fn poll(&mut self) -> Option<ci::build::Work> {
        // Push a pending job if running jobs have no work.
        if !self.running_jobs.iter().any(|job| job.has_pending_tasks()) {
            if let Some(pending_job) = self.pending_jobs.pop_front() {
                // We may need to move the next job onto the queue.
                self.running_jobs.push_back(RunningJob {
                    pending_tasks: pending_job.tasks.iter().cloned().collect(),
                    running_work: HashMap::new(),
                    completed_tasks: Vec::new(),
                    job: pending_job,
                });
            }
        }

        if let Some(running_job) = self.running_jobs.iter_mut().find(|job| job.has_pending_tasks()) {
            let tasks = if let Some(pending_task) = running_job.pending_tasks.pop_front() {
                vec![pending_task]
            } else {
                vec![]
            };

            let work = ci::build::Work {
                uuid: Uuid::new_v4(),
                tasks: tasks.into_iter().collect(),
            };

            let running_work = RunningWork {
                uuid: work.uuid.clone(),
                running_tasks: work.tasks.iter().map(|t| (t.uuid.clone(), t.clone())).collect(),
                completed_tasks: Vec::new(),
            };

            running_job.running_work.insert(running_work.uuid.clone(), running_work);

            Some(work)
        } else {
            None
        }
    }

    /// Marks some work as completed.
    pub fn complete(&mut self, work: CompletedWork) {
        {
            let job_uuid = self.find_job_uuid_containing_work_uuid(&work.uuid).unwrap();
            let running_job = self.running_jobs.iter_mut().find(|job| job.job.uuid == job_uuid).unwrap();
            let mut running_work = running_job.running_work.remove(&work.uuid).unwrap();

            running_work.completed_tasks.extend(work.task_results);
            running_job.completed_tasks.extend(running_work.completed_tasks);
        }

        self.move_finished_jobs();
    }

    /// Checks if the dispatcher has work ready.
    pub fn has_work(&self) -> bool {
        !self.pending_jobs.is_empty() || !self.running_jobs.is_empty()
    }

    pub fn completed_jobs(&mut self) -> impl Iterator<Item=CompletedJob> {
        let completed_jobs: Vec<_> = self.completed_jobs.drain(..).collect();
        completed_jobs.into_iter()
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

    pub fn has_pending_tasks(&self) -> bool {
        !self.pending_tasks.is_empty()
    }
}

#[cfg(test)]
mod test
{
    pub use super::*;
    pub use ci::job::*;
    pub use ci::build::*;
    pub use uuid::Uuid;

    fn setup() -> (Dispatcher, Job, Job, Task, Task) {
        let task1 = Task {
            uuid: Uuid::new_v4(),
            command: Command {
                executable: "echo".to_owned(),
                arguments: vec!["foo".to_owned(), "bar".to_owned()],
            },
        };

        let task2 = Task {
            uuid: Uuid::new_v4(),
            command: Command {
                executable: "cat".to_owned(),
                arguments: vec!["/etc/hosts".to_owned()],
            },
        };

        let job1 = Job {
            uuid: Uuid::new_v4(),
            tasks: vec![task1.clone()],
        };

        let job2 = Job {
            uuid: Uuid::new_v4(),
            tasks: vec![task2.clone()],
        };

        let mut dispatcher = Dispatcher::new();
        dispatcher.enqueue(job1.clone());
        assert_eq!(dispatcher.pending_jobs.len(), 1);
        dispatcher.enqueue(job2.clone());
        assert_eq!(dispatcher.pending_jobs.len(), 2);

        (dispatcher, job1, job2, task1, task2)
    }

    #[test]
    fn enqueue_correctly_adds_jobs() {
        let (mut dispatcher, job1, _, _, _) = setup();

        assert_eq!(dispatcher.pending_jobs.len(), 2);
        dispatcher.enqueue(job1);
        assert_eq!(dispatcher.pending_jobs.len(), 3);
    }

    #[test]
    fn poll_works_in_correct_order() {
        let (mut dispatcher, _, _, task1, task2) = setup();

        assert_eq!(dispatcher.poll().unwrap().tasks[0], task1);
        assert_eq!(dispatcher.poll().unwrap().tasks[0], task2);
        assert_eq!(dispatcher.poll(), None);
        assert_eq!(dispatcher.poll(), None);
    }

    #[test]
    fn complete_works_as_expected() {
        let (mut dispatcher, _, _, _, _) = setup();

        assert!(dispatcher.has_work());

        assert_eq!(dispatcher.running_jobs.len(), 0);
        let work = dispatcher.poll().unwrap();
        assert_eq!(dispatcher.running_jobs.len(), 1);

        dispatcher.complete(CompletedWork {
            uuid: work.uuid,
            task_results: work.tasks.into_iter().map(|task| {
                TaskResult {
                    task: task,
                    output: TaskOutput {
                        output: Vec::new(),
                        result_code: 0,
                    },
                }
            }).collect(),
        });

        assert_eq!(dispatcher.running_jobs.len(), 0);
        assert!(dispatcher.has_work());

        let work = dispatcher.poll().unwrap();
        assert_eq!(dispatcher.running_jobs.len(), 1);
        assert_eq!(dispatcher.pending_jobs.len(), 0);

        dispatcher.complete(CompletedWork {
            uuid: work.uuid,
            task_results: work.tasks.into_iter().map(|task| {
                TaskResult {
                    task: task,
                    output: TaskOutput {
                        output: Vec::new(),
                        result_code: 0,
                    },
                }
            }).collect(),
        });

        assert_eq!(dispatcher.running_jobs.len(), 0);
        assert_eq!(dispatcher.pending_jobs.len(), 0);
        assert!(!dispatcher.has_work());
    }
}

