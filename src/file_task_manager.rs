pub trait TaskRepository {
    fn get_all(&mut self) -> &Vec<Task>;
}

use std::path::PathBuf;
use crate::entity::task::Task;

#[derive(Debug)]
pub struct FileTaskRepository {
    tasks: Option<Vec<Task>>,
    file_path: PathBuf,
}

impl FileTaskRepository {
    /// Creates a new `FileTaskRepository` with the specified file path.
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            tasks: None,
            file_path,
        }
    }

    /// Loads tasks from the file if they have not been loaded yet.
    fn load_tasks_from_file(&mut self) {
        if self.tasks.is_some() {
            return;
        }

        match std::fs::read_to_string(&self.file_path) {
            Ok(content) => {
                match serde_json::from_str::<Vec<Task>>(&content) {
                    Ok(parsed_tasks) => {
                        self.tasks = Some(parsed_tasks);
                    }
                    Err(e) => {
                        eprintln!("Error parsing JSON file: {}", e);
                        self.tasks = Some(vec![]);
                    }
                }
            }
            Err(e) => {
                eprintln!("Unable to read tasks file: {}", e);
                self.tasks = Some(vec![]);
            }
        }
    }
}

impl TaskRepository for FileTaskRepository {
    fn get_all(&mut self) -> &Vec<Task> {
        self.load_tasks_from_file();
        self.tasks.as_ref().unwrap()
    }
}
