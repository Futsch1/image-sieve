use std::{fs, path::Path};

use super::settings::Settings;
use crate::item_sort_list::ItemList;
use home;

/// Name of the global settings file
const SETTINGS_FILE: &str = "image_sieve_settings.json";

/// Name of the project settings file
const ITEM_LIST_FILE: &str = "image_sieve.json";

pub fn get_settings_filename() -> String {
    let home = home::home_dir();
    if let Some(home) = home {
        if !Path::new(&home.join(".image_sieve")).exists() {
            fs::create_dir_all(&home.join(".image_sieve")).unwrap();
        }
        home.join(".image_sieve")
            .join(SETTINGS_FILE)
            .to_str()
            .unwrap()
            .to_string()
    } else {
        String::from(SETTINGS_FILE)
    }
}

pub fn get_project_filename(path: &str) -> String {
    let path = Path::new(path).to_path_buf().join(ITEM_LIST_FILE);
    String::from(path.to_str().unwrap())
}

pub trait JsonPersistence
where
    Self: Sized,
{
    fn load(file_name: &str) -> Option<Self>;
    fn save(file_name: &str, object: &Self);
}

impl JsonPersistence for Settings {
    /// Construct a new Settings struct by loading the data from a json file
    fn load(file_name: &str) -> Option<Settings> {
        let settings = fs::read_to_string(file_name).unwrap_or_default();

        let contents = serde_json::from_str::<Settings>(&settings);
        if let Ok(settings) = contents {
            Some(settings)
        } else {
            None
        }
    }

    /// Try saving the settings to a json file
    fn save(file_name: &str, settings: &Settings) {
        let settings = serde_json::to_string_pretty(settings).unwrap_or_default();
        fs::write(file_name, settings).ok();
    }
}

impl JsonPersistence for ItemList {
    fn load(file_name: &str) -> Option<ItemList> {
        let settings = fs::read_to_string(file_name).unwrap_or_default();

        let contents = serde_json::from_str::<ItemList>(&settings);
        if let Ok(settings) = contents {
            Some(settings)
        } else {
            None
        }
    }

    fn save(file_name: &str, item_list: &ItemList) {
        let item_list = serde_json::to_string_pretty(item_list).unwrap_or_default();
        fs::write(file_name, item_list).ok();
    }
}
