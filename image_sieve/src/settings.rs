use item_sort_list::CommitMethod;
use std::env;

pub struct Settings {
    pub source_directory: String,
    pub target_directory: String,
    pub commit_method: CommitMethod,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            source_directory: String::from(
                env::current_dir().unwrap_or_default().to_str().unwrap(),
            ),
            target_directory: String::from(
                env::current_dir().unwrap_or_default().to_str().unwrap(),
            ),
            commit_method: CommitMethod::Copy,
        }
    }
}
