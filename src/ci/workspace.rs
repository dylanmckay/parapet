use ci;

use std::path::PathBuf;
use std::collections::HashMap;

/// A ci of projects.
pub struct Workspace
{
    path: PathBuf,
    projects: HashMap<String, ci::Project>,
}

impl Workspace
{
    pub fn new(path: PathBuf) -> Self {
        Workspace {
            path: path,
            projects: HashMap::new(),
        }
    }

    pub fn open_project(&mut self, name: String) -> &mut ci::Project {
        let project_path = self.path.join(name.clone());
        self.projects.insert(name.clone(), ci::Project::new(name.clone(), project_path));
        self.projects.get_mut(&name).unwrap()
    }
}

