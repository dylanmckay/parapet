use workspace;

use std::path::PathBuf;
use std::collections::HashMap;

/// A workspace of projects.
pub struct Workspace
{
    path: PathBuf,
    projects: HashMap<String, workspace::Project>,
}

impl Workspace
{
    pub fn new(path: PathBuf) -> Self {
        Workspace {
            path: path,
            projects: HashMap::new(),
        }
    }

    pub fn open_project(&mut self, name: String) -> &mut workspace::Project {
        let project_path = self.path.join(name.clone());
        self.projects.insert(name.clone(), workspace::Project::new(name.clone(), project_path));
        self.projects.get_mut(&name).unwrap()
    }
}

