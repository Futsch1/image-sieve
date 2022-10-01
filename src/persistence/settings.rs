use crate::item_sort_list::{DirectoryNames, SieveMethod};
use crate::main_window::{ImageSieve, SieveComboValues};
use serde::{Deserialize, Serialize};
use slint::{ComponentHandle, ModelRc, PhysicalPosition, PhysicalSize, SharedString};

use super::model_to_enum::{enum_to_model, model_to_enum};

#[derive(Serialize, Deserialize, std::fmt::Debug, PartialEq, Eq)]
pub struct SettingsV05 {
    pub source_directory: String,
    pub target_directory: String,
    pub sieve_method: SieveMethod,
    pub use_timestamps: bool,
    pub timestamp_max_diff: i64,
    pub use_hash: bool,
    pub hash_max_diff: u32,
    pub sieve_directory_names: Option<DirectoryNames>,
    pub dark_mode: String,
}

#[derive(Serialize, Deserialize, std::fmt::Debug, PartialEq, Eq)]
pub struct SettingsV06 {
    pub width: u32,
    pub height: u32,
    pub left: i32,
    pub top: i32,
}

#[derive(std::fmt::Debug, PartialEq, Eq)]
pub struct Settings {
    pub settings_v05: SettingsV05,
    pub settings_v06: SettingsV06,
}

impl SettingsV05 {
    pub fn new() -> Self {
        Self {
            source_directory: String::new(),
            target_directory: String::new(),
            sieve_method: SieveMethod::Copy,
            use_timestamps: true,
            timestamp_max_diff: 5,
            use_hash: false,
            hash_max_diff: 14,
            sieve_directory_names: Some(DirectoryNames::YearAndMonth),
            dark_mode: String::from("Automatic"),
        }
    }
}

impl SettingsV06 {
    pub fn new() -> Self {
        Self {
            width: 1600,
            height: 800,
            left: 100,
            top: 100,
        }
    }
}

impl Settings {
    pub fn new() -> Self {
        Self {
            settings_v05: SettingsV05::new(),
            settings_v06: SettingsV06::new(),
        }
    }

    pub fn from_window(window: &ImageSieve) -> Self {
        //TODO: Also save last selected image and restart there
        let methods: ModelRc<SharedString> = window.global::<SieveComboValues>().get_methods();
        let directory_names: ModelRc<SharedString> =
            window.global::<SieveComboValues>().get_directory_names();
        let settings_v05 = SettingsV05 {
            source_directory: window.get_source_directory().to_string(),
            target_directory: window.get_target_directory().to_string(),
            sieve_method: model_to_enum(&methods, &window.get_sieve_method()),
            use_timestamps: window.get_use_timestamps(),
            timestamp_max_diff: convert_timestamp_difference(&window.get_timestamp_difference())
                .unwrap_or(5),
            use_hash: window.get_use_similarity(),
            hash_max_diff: convert_sensitivity_to_u32(&window.get_similarity_sensitivity()),
            sieve_directory_names: Some(model_to_enum(
                &directory_names,
                &window.get_sieve_directory_names(),
            )),
            dark_mode: window.get_dark_mode().to_string(),
        };
        let position = window.window().position();
        let size = window.window().size();
        let settings_v06 = SettingsV06 {
            width: size.width,
            height: size.height,
            left: position.x,
            top: position.y,
        };
        Settings {
            settings_v05,
            settings_v06,
        }
    }

    pub fn to_window(&self, window: &ImageSieve) {
        window.set_source_directory(SharedString::from(
            self.settings_v05.source_directory.clone(),
        ));
        window.set_target_directory(SharedString::from(
            self.settings_v05.target_directory.clone(),
        ));
        let methods: ModelRc<SharedString> = window.global::<SieveComboValues>().get_methods();
        window.set_sieve_method(enum_to_model(&methods, &self.settings_v05.sieve_method));
        window.set_use_timestamps(self.settings_v05.use_timestamps);
        window.set_timestamp_difference(SharedString::from(
            self.settings_v05.timestamp_max_diff.to_string(),
        ));
        window.set_use_similarity(self.settings_v05.use_hash);
        window.set_similarity_sensitivity(SharedString::from(convert_u32_to_sensitivity(
            self.settings_v05.hash_max_diff,
        )));
        let directory_names: ModelRc<SharedString> =
            window.global::<SieveComboValues>().get_directory_names();
        let directory_name = self
            .settings_v05
            .sieve_directory_names
            .as_ref()
            .unwrap_or(&DirectoryNames::YearAndMonth);
        window.set_sieve_directory_names(enum_to_model(&directory_names, directory_name));
        window.set_dark_mode(SharedString::from(self.settings_v05.dark_mode.clone()));
        window.window().set_position(PhysicalPosition::new(
            self.settings_v06.left,
            self.settings_v06.top,
        ));
        window.window().set_size(PhysicalSize::new(
            self.settings_v06.width,
            self.settings_v06.height,
        ))
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
        17.. => "Very low",
        15..=16 => "Low",
        13..=14 => "Medium",
        11..=12 => "High",
        0..=10 => "Very high",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers() {
        assert_eq!(convert_timestamp_difference("5"), Some(5));
        assert_eq!(convert_timestamp_difference("x"), None);

        assert_eq!(convert_sensitivity_to_u32("Very low"), 20);
        assert_eq!(convert_sensitivity_to_u32("Very high"), 10);
        assert_eq!(
            convert_sensitivity_to_u32("Something"),
            convert_sensitivity_to_u32("Medium")
        );

        assert_eq!(convert_u32_to_sensitivity(20), "Very low");
        assert_eq!(convert_u32_to_sensitivity(40), "Very low");
        assert_eq!(convert_u32_to_sensitivity(10), "Very high");
        assert_eq!(convert_u32_to_sensitivity(0), "Very high");
        assert_eq!(convert_u32_to_sensitivity(11), "High");
    }

    #[test]
    fn from_to_window() {
        let window = ImageSieve::new();

        let settings = Settings::new();
        let settings2 = Settings::from_window(&window);
        assert_ne!(settings, settings2);

        settings.to_window(&window);
        let settings3 = Settings::from_window(&window);
        assert_eq!(settings, settings3);
    }
}
