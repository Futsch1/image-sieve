extern crate chrono;
extern crate glob;

use self::chrono::NaiveDateTime;
use num_derive::{FromPrimitive, ToPrimitive};
use std::path::Path;

use super::event;
use super::file_item;
use super::resolvers;

#[derive(PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum CommitMethod {
    Copy = 0,
    Move,
    MoveAndDelete,
    Delete,
}

#[derive(Clone)]
pub struct ItemList {
    pub items: Vec<file_item::FileItem>,
    pub events: Vec<event::Event>,
    pub path: String,
}

impl ItemList {
    /// Synchronize an existing items list with the items found in a path
    pub fn synchronize(&mut self, path: &str) {
        let mut found_item_paths = find_items(path);

        self.items = self
            .items
            .drain(..)
            .filter(|i| i.get_path().exists() && found_item_paths.contains(i.get_path_as_str()))
            .collect();

        // Add all newly found
        for item_path in found_item_paths.drain(..) {
            let item = Self::create_item(item_path, true);
            let path = item.get_path();
            if !self.items.iter().any(|i| i.get_path() == path) {
                self.items.push(item);
            }
        }
        self.items.sort();
        self.path = String::from(path);
    }

    /// Adds an item to the list
    pub fn add_item(&mut self, item_path: String, take_over: bool) {
        self.items.push(Self::create_item(item_path, take_over));
    }

    /// Internal function to create a new file item
    fn create_item(item_path: String, take_over: bool) -> file_item::FileItem {
        let resolver = resolvers::get_resolver(&item_path);
        file_item::FileItem::new(item_path, resolver, take_over, None)
    }

    /// Go through all images and find similar ones by comparing the timestamp
    pub fn find_similar(&mut self, max_diff_seconds: i64, max_diff_hash: u32) {
        for item in self.items.iter_mut() {
            item.calc_hash();
        }

        for index in 0..self.items.len() {
            for other_index in index + 1..self.items.len() {
                if other_index != index
                    && self.items[index].is_similar(
                        &self.items[other_index],
                        max_diff_seconds,
                        max_diff_hash,
                    )
                {
                    self.items[index].add_similar(other_index);
                    self.items[other_index].add_similar(index);
                }
            }
        }
    }

    /// Commits an item list taking the take_over flag into account to a new directory.
    /// The progress is reported by calling a callback function with the file that is currently processed.
    pub fn commit(
        &self,
        path: &str,
        commit_method: CommitMethod,
        progress_callback: impl Fn(String),
    ) {
        // TODO: Refactor maybe
        use std::fs::{copy, remove_file, rename};

        if commit_method != CommitMethod::Delete {
            let path = Path::new(path);
            prepare_path(path);

            for item in &self.items {
                if item.get_take_over() {
                    let full_path = path.join(self.get_sub_path(item));
                    prepare_path(&full_path);
                    let source = item.get_path();
                    let target = full_path.join(source.file_name().unwrap());
                    let mut operation = String::from(source.to_str().unwrap());
                    operation += " -> ";
                    operation += target.to_str().unwrap();
                    progress_callback(operation);

                    if commit_method == CommitMethod::Copy {
                        match copy(source, target) {
                            Ok(_) => (),
                            Err(e) => println!("Error copying {:?}: {}", item, e),
                        }
                    } else {
                        match rename(source, target) {
                            Ok(_) => (),
                            Err(e) => println!("Error renaming {:?}: {}", item, e),
                        }
                    };
                } else if commit_method == CommitMethod::MoveAndDelete {
                    let source = item.get_path();
                    let mut operation = String::from("Delete ");
                    operation += source.to_str().unwrap();
                    progress_callback(operation);
                    match remove_file(source) {
                        Ok(_) => (),
                        Err(e) => println!("Error deleting {:?}: {}", item, e),
                    }
                }
            }
        } else {
            for item in &self.items {
                if !item.get_take_over() {
                    let source = item.get_path();
                    let mut operation = String::from("Delete ");
                    operation += source.to_str().unwrap();
                    progress_callback(operation);
                    match remove_file(source) {
                        Ok(_) => (),
                        Err(e) => println!("Error deleting {:?}: {}", item, e),
                    }
                }
            }
        }

        progress_callback(String::from("Done"));
    }

    pub fn get_sub_path(&self, item: &file_item::FileItem) -> String {
        let event = self.get_event(item);
        if let Some(event) = event {
            if event.start_date != event.end_date {
                return format!(
                    "{} - {} {}",
                    event.start_date.format("%Y-%m-%d"),
                    event.end_date.format("%Y-%m-%d"),
                    event.name
                );
            } else {
                return format!("{} {}", event.start_date.format("%Y-%m-%d"), event.name);
            }
        }
        NaiveDateTime::from_timestamp(item.get_timestamp(), 0)
            .format("%Y-%m")
            .to_string()
    }

    pub fn get_event(&self, item: &file_item::FileItem) -> Option<&event::Event> {
        let naive_date = NaiveDateTime::from_timestamp(item.get_timestamp(), 0).date();
        for event in &self.events {
            if event.start_date <= naive_date && naive_date <= event.end_date {
                return Some(event);
            }
        }
        None
    }
}

fn prepare_path(path: &Path) {
    use std::fs::create_dir_all;

    if !path.exists() {
        match create_dir_all(path) {
            Ok(_) => (),
            Err(e) => println!("Error creating path {}: {}", e, path.display()),
        }
    }
}

fn find_items(path: &str) -> Vec<String> {
    let match_options = glob::MatchOptions {
        case_sensitive: false,
        require_literal_leading_dot: false,
        require_literal_separator: false,
    };
    let mut files: Vec<String> = Vec::new();

    for extension in file_item::FileItem::get_extensions() {
        let glob_pattern = format!("{}/**/*.{}", path, extension);
        let entries = glob::glob_with(&glob_pattern, match_options).unwrap();
        for entry in entries {
            let jpg: std::path::PathBuf = entry.expect("IO error");
            files.push(jpg.to_str().unwrap().to_string());
        }
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item_sort_list::item_traits::PropertyResolver;

    static mut CALL_COUNT: usize = 0;

    struct MockResolver {}

    impl PropertyResolver for MockResolver {
        fn get_timestamp(&self) -> i64 {
            let return_values: [i64; 6] = [1, 4, 8, 14, 64, 65];
            unsafe {
                CALL_COUNT += 1;
                return_values[CALL_COUNT - 1]
            }
        }

        fn get_orientation(&self) -> Option<crate::item_sort_list::Orientation> {
            None
        }
    }

    #[test]
    fn find_similar() {
        let mut items: Vec<file_item::FileItem> = vec![];
        for _ in 0..6 {
            items.push(file_item::FileItem::new(
                String::from(""),
                Box::new(MockResolver {}),
                true,
                None,
            ));
        }
        let mut item_list = ItemList {
            items,
            events: vec![],
            path: String::from(""),
        };

        item_list.find_similar(5, 6);

        assert_eq!(2, item_list.items[0].get_similars().len());
        assert_eq!(2, item_list.items[1].get_similars().len());
        assert_eq!(2, item_list.items[2].get_similars().len());
        assert_eq!(0, item_list.items[3].get_similars().len());
        assert_eq!(1, item_list.items[4].get_similars().len());
        assert_eq!(1, item_list.items[5].get_similars().len());
    }

    #[test]
    fn get_sub_path() {
        use self::chrono::NaiveDate;
        use self::chrono::NaiveDateTime;

        let item_list = ItemList {
            items: vec![],
            events: vec![
                event::Event {
                    name: String::from("Test1"),
                    start_date: NaiveDate::from_ymd(2021, 9, 14),
                    end_date: NaiveDate::from_ymd(2021, 9, 14),
                },
                event::Event {
                    name: String::from("Test2"),
                    start_date: NaiveDate::from_ymd(2021, 9, 20),
                    end_date: NaiveDate::from_ymd(2021, 9, 21),
                },
                event::Event {
                    name: String::from("Test3"),
                    start_date: NaiveDate::from_ymd(2021, 9, 24),
                    end_date: NaiveDate::from_ymd(2021, 9, 27),
                },
            ],
            path: String::from(""),
        };
        let test_cases = [
            ("2021-09-14 00:00", "2021-09-14 Test1"),
            ("2021-09-13 23:59", "2021-09"),
            ("2021-09-15 00:00", "2021-09"),
            ("2021-09-20 00:00", "2021-09-20 - 2021-09-21 Test2"),
            ("2021-09-21 16:00", "2021-09-20 - 2021-09-21 Test2"),
            ("2021-09-25 16:00", "2021-09-24 - 2021-09-27 Test3"),
        ];

        for (input, result) in test_cases {
            assert_eq!(
                item_list.get_sub_path(&file_item::FileItem::dummy(
                    NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M")
                        .unwrap()
                        .timestamp()
                )),
                result
            );
        }
    }
}
