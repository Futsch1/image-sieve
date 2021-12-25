use crate::item_sort_list::SieveMethod;
use crate::main_window::{ImageSieve, SieveMethodValues};
use num_traits::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use sixtyfps::{ComponentHandle, Model, ModelHandle, SharedString};

#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
    pub source_directory: String,
    pub target_directory: String,
    pub sieve_method: SieveMethod,
    pub use_timestamps: bool,
    pub timestamp_max_diff: i64,
    pub use_hash: bool,
    pub hash_max_diff: u32,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            source_directory: String::new(),
            target_directory: String::new(),
            sieve_method: SieveMethod::Copy,
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
            sieve_method: FromPrimitive::from_i32(window.get_sieve_method())
                .unwrap_or(SieveMethod::Copy),
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
        let sieve_index = ToPrimitive::to_i32(&self.sieve_method).unwrap();
        window.set_sieve_method(sieve_index);
        let values: ModelHandle<SharedString> = window.global::<SieveMethodValues>().get_values();
        window.set_sieve_method_value(values.row_data(sieve_index as usize));
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
        "Very low" => 20,
        "Low" => 16,
        "Medium" => 14,
        "High" => 12,
        "Very high" => 10,
        _ => 14,
    }
}

fn convert_u32_to_sensitivity(sensitivity: u32) -> &'static str {
    match sensitivity {
        20 => "Very low",
        16 => "Low",
        14 => "Medium",
        12 => "High",
        10 => "Very high",
        _ => "Medium",
    }
}
