use crate::item_sort_list::CommitMethod;
use std::env;

pub struct Settings {
    pub source_directory: String,
    pub target_directory: String,
    pub commit_method: CommitMethod,
    pub use_timestamps: bool,
    pub timestamp_max_diff: u64,
    pub use_hash: bool,
    pub hash_max_diff: u32,
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
            use_timestamps: true,
            timestamp_max_diff: 5,
            use_hash: false,
            hash_max_diff: 20,
        }
    }
}
