use crate::item_sort_list::CommitMethod;
use crate::main_window::{CommitMethodValues, ImageSieve};
use num_traits::{FromPrimitive, ToPrimitive};
use sixtyfps::{ComponentHandle, Model, ModelHandle, SharedString};
use std::env;

#[derive(Clone)]
pub struct Settings {
    pub source_directory: String,
    pub target_directory: String,
    pub commit_method: CommitMethod,
    pub use_timestamps: bool,
    pub timestamp_max_diff: i64,
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
            hash_max_diff: 8,
        }
    }

    pub fn from_window(window: &ImageSieve) -> Self {
        //TODO: Also save last selected image and restart there
        Settings {
            source_directory: window.get_source_directory().to_string(),
            target_directory: window.get_target_directory().to_string(),
            commit_method: FromPrimitive::from_i32(window.get_commit_method())
                .unwrap_or(CommitMethod::Copy),
            use_timestamps: window.get_use_timestamps(),
            timestamp_max_diff: convert_timestamp_difference(
                &window.get_timestamp_difference().to_string(),
            )
            .unwrap_or(5),
            use_hash: window.get_use_similarity(),
            hash_max_diff: convert_sensitivity_to_u32(
                &window.get_similarity_sensitivity().to_string(),
            ),
        }
    }

    pub fn to_window(&self, window: &ImageSieve) {
        window.set_source_directory(SharedString::from(self.source_directory.clone()));
        window.set_target_directory(SharedString::from(self.target_directory.clone()));
        let commit_index = ToPrimitive::to_i32(&self.commit_method).unwrap();
        window.set_commit_method(commit_index);
        let values: ModelHandle<SharedString> = window.global::<CommitMethodValues>().get_values();
        window.set_commit_method_value(values.row_data(commit_index as usize));
        window.set_use_timestamps(self.use_timestamps);
        window.set_timestamp_difference(SharedString::from(self.timestamp_max_diff.to_string()));
        window.set_use_similarity(self.use_hash);
        window.set_similarity_sensitivity(SharedString::from(convert_u32_to_sensitivity(
            self.hash_max_diff,
        )));
    }
}

fn convert_timestamp_difference(timestamp_difference: &str) -> Option<i64> {
    if let Ok(timestamp_difference) = timestamp_difference.parse::<i64>() {
        Some(timestamp_difference)
    } else {
        None
    }
}

fn convert_sensitivity_to_u32(sensitivity: &str) -> u32 {
    match sensitivity {
        "Very low" => 10,
        "Low" => 8,
        "Medium" => 7,
        "High" => 6,
        "Very high" => 5,
        _ => 10,
    }
}

fn convert_u32_to_sensitivity(sensitivity: u32) -> &'static str {
    match sensitivity {
        10 => "Very low",
        8 => "Low",
        7 => "Medium",
        6 => "High",
        5 => "Very high",
        _ => "Medium",
    }
}
