use std::{
    fs,
    path::{Path, PathBuf},
};

use super::settings::Settings;
use crate::item_sort_list::ItemList;
use home;

/// Name of the global settings file
const SETTINGS_FILE: &str = "image_sieve_settings.json";

/// Name of the project settings file
const ITEM_LIST_FILE: &str = "image_sieve.json";

/// Get the directory and filename where the settings are stored
pub fn get_settings_filename() -> PathBuf {
    let home = home::home_dir();
    if let Some(home) = home {
        if !Path::new(&home.join(".image_sieve")).exists() {
            fs::create_dir_all(&home.join(".image_sieve")).unwrap();
        }
        home.join(".image_sieve").join(SETTINGS_FILE)
    } else {
        PathBuf::from(SETTINGS_FILE)
    }
}

/// Get the directory and filename where the item list is stored
pub fn get_project_filename(path: &Path) -> PathBuf {
    Path::new(path).to_path_buf().join(ITEM_LIST_FILE)
}

/// Trait to load and save data from/to a file
pub trait JsonPersistence
where
    Self: Sized,
{
    fn load(file_name: &Path) -> Option<Self>;
    fn save(file_name: &Path, object: &Self);
}

impl JsonPersistence for Settings {
    /// Construct a new Settings struct by loading the data from a json file
    fn load(file_name: &Path) -> Option<Settings> {
        let settings = fs::read_to_string(file_name).unwrap_or_default();

        let contents = serde_json::from_str::<Settings>(&settings);
        if let Ok(settings) = contents {
            Some(settings)
        } else {
            None
        }
    }

    /// Try saving the settings to a json file
    fn save(file_name: &Path, settings: &Settings) {
        let settings = serde_json::to_string_pretty(settings).unwrap_or_default();
        fs::write(file_name, settings).ok();
    }
}

impl JsonPersistence for ItemList {
    fn load(file_name: &Path) -> Option<ItemList> {
        let item_list = fs::read_to_string(file_name).unwrap_or_default();

        let contents = serde_json::from_str::<ItemList>(&item_list);
        if let Ok(item_list) = contents {
            Some(item_list)
        } else {
            None
        }
    }

    fn save(file_name: &Path, item_list: &ItemList) {
        let item_list = serde_json::to_string_pretty(item_list).unwrap_or_default();
        fs::write(file_name, item_list).ok();
    }
}
