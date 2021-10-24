use json::JsonValue;
use num_traits::{FromPrimitive, ToPrimitive};
use std::{fs, path::Path};

use super::settings::Settings;
use crate::item_sort_list::{Event, ItemList, EVENT_DATE_FORMAT};

/// Name of the global settings file
const SETTINGS_FILE: &str = "settings.json";

/// Name of the project settings file
const ITEM_LIST_FILE: &str = "image_sieve.json";

pub fn get_settings_filename() -> &'static str {
    SETTINGS_FILE
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
        let contents = json::parse(settings.as_str());
        match contents {
            Ok(json) => Some(Settings {
                source_directory: json["source_directory"].to_string(),
                target_directory: json["target_directory"].to_string(),
                commit_method: FromPrimitive::from_i32(json["commit_method"].as_i32().unwrap_or(0))
                    .unwrap(),
                use_timestamps: json["use_timestamps"].as_bool().unwrap_or(true),
                timestamp_max_diff: json["timestamp_max_diff"].as_u64().unwrap_or(5),
                use_hash: json["use_hash"].as_bool().unwrap_or(false),
                hash_max_diff: json["hash_max_diff"].as_u32().unwrap_or(15),
            }),
            _ => None,
        }
    }

    /// Try saving the settings to a json file
    fn save(file_name: &str, settings: &Settings) {
        let json = json::object! {source_directory: settings.source_directory.clone(),
        target_directory: settings.target_directory.clone(), commit_method: ToPrimitive::to_i32(&settings.commit_method)};
        fs::write(file_name, json::stringify_pretty(json, 4)).ok();
    }
}

impl JsonPersistence for ItemList {
    fn load(file_name: &str) -> Option<ItemList> {
        let settings = fs::read_to_string(file_name).unwrap_or_default();
        let contents = json::parse(settings.as_str());
        match contents {
            Ok(json) => {
                let mut item_list = ItemList {
                    items: vec![],
                    events: vec![],
                    path: json["path"].to_string(),
                };
                if let JsonValue::Array(json_item_list) = &json["item_list"] {
                    for json_item in json_item_list {
                        // Only add item if it still exists

                        let item_path = json_item["file_name"].to_string();
                        if std::path::Path::new(&item_path).exists() {
                            item_list.add_item(
                                item_path,
                                json_item["take_over"].as_bool().unwrap_or(true),
                            );
                        }
                    }
                }
                if let JsonValue::Array(json_event_list) = &json["event_list"] {
                    for json_event in json_event_list {
                        let event = Event::new(
                            json_event["name"].to_string(),
                            json_event["start_date"].as_str().unwrap(),
                            json_event["end_date"].as_str().unwrap(),
                        );
                        if let Ok(event) = event {
                            item_list.events.push(event);
                        }
                    }
                }

                Some(item_list)
            }
            _ => None,
        }
    }

    fn save(file_name: &str, item_list: &ItemList) {
        let mut json_file_list = JsonValue::new_array();
        for file_item in &item_list.items {
            let json_file_item = json::object! {file_name: file_item.get_path().to_str(), take_over: file_item.get_take_over()};
            json_file_list.push(json_file_item).ok();
        }

        let mut json_event_list = JsonValue::new_array();
        for event in &item_list.events {
            let json_event_item = json::object! {
                name: event.name.clone(),
                start_date: event.start_date.format(EVENT_DATE_FORMAT).to_string(),
                end_date: event.end_date.format(EVENT_DATE_FORMAT).to_string(),
            };
            json_event_list.push(json_event_item).ok();
        }

        let json = json::object! {item_list: json_file_list, event_list: json_event_list, path: item_list.path.clone()};
        fs::write(file_name, json::stringify_pretty(json, 4)).ok();
    }
}
