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

/// Name of the trace file
const TRACE_FILE: &str = "trace.txt";

/// Get the directory and filename where traces are stored
pub fn get_trace_filename() -> PathBuf {
    get_and_create_home_dir().join(TRACE_FILE)
}

/// Get the directory and filename where the settings are stored
pub fn get_settings_filename() -> PathBuf {
    get_and_create_home_dir().join(SETTINGS_FILE)
}

/// Get the directory and filename where the item list is stored
pub fn get_project_filename(path: &Path) -> PathBuf {
    Path::new(path).to_path_buf().join(ITEM_LIST_FILE)
}

fn get_and_create_home_dir() -> PathBuf {
    let home = home::home_dir();
    if let Some(home) = home {
        if !Path::new(&home.join(".image_sieve")).exists() {
            fs::create_dir_all(home.join(".image_sieve")).unwrap();            
        }
        home.join(".image_sieve")
    } else {
        PathBuf::from(".")
    }
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
        if let Ok(mut item_list) = contents {
            for file_item in &mut item_list.items {
                file_item.deserialized();
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item_sort_list::Event;
    use crate::item_sort_list::FileItem;
    use crate::item_sort_list::{DirectoryNames, SieveMethod};
    use chrono::NaiveDate;
    use img_hash::ImageHash;

    #[test]
    fn test_get_names() {
        assert!(!get_settings_filename().as_os_str().is_empty());
        let project_filename = get_project_filename(Path::new("test"));
        let project_filename_str = project_filename.to_str().unwrap();
        assert!(project_filename_str.contains("test"));
        assert!(project_filename_str.contains(ITEM_LIST_FILE));
        assert!(!get_trace_filename().as_os_str().is_empty());
    }

    #[test]
    fn test_load_save_item_list() {
        let mut item_list = ItemList {
            items: vec![
                FileItem::dummy("test/test1.jpg", 0, true),
                FileItem::dummy("test/test2.jpg", 0, false),
            ],
            events: vec![Event {
                name: String::from("Test1"),
                start_date: NaiveDate::from_ymd_opt(2021, 9, 14).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2021, 9, 14).unwrap(),
            }],
            path: PathBuf::from("test"),
        };
        let hash = ImageHash::<Vec<u8>>::from_bytes(&[0x64, 0x65, 0x66, 0x67])
            .unwrap()
            .to_base64();
        item_list.items[0].set_encoded_hash(&hash);

        JsonPersistence::save(Path::new("test_il.json"), &item_list);

        let loaded_item_list: ItemList = JsonPersistence::load(Path::new("test_il.json")).unwrap();
        assert_eq!(loaded_item_list.path, item_list.path);
        assert_eq!(loaded_item_list.events, item_list.events);
        assert_eq!(loaded_item_list.items, item_list.items);

        let loaded_item_list: Option<ItemList> = JsonPersistence::load(Path::new("invalid.json"));
        assert!(loaded_item_list.is_none());
    }

    #[test]
    fn test_load_save_settings() {
        let mut settings = Settings::new();
        settings.source_directory += "source";
        settings.target_directory += "target";
        settings.sieve_method = SieveMethod::MoveAndDelete;
        settings.use_timestamps = !settings.use_timestamps;
        settings.timestamp_max_diff += 1;
        settings.use_hash = !settings.use_hash;
        settings.hash_max_diff = 12;
        settings.sieve_directory_names = Some(DirectoryNames::YearAndQuarter);
        settings.dark_mode = String::from("On");

        JsonPersistence::save(Path::new("test.json"), &settings);

        let loaded_settings = JsonPersistence::load(Path::new("test.json")).unwrap();
        assert_eq!(settings, loaded_settings);

        let loaded_settings: Option<Settings> = JsonPersistence::load(Path::new("invalid.json"));
        assert!(loaded_settings.is_none());
    }
}
