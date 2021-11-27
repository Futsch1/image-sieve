extern crate chrono;
extern crate glob;

use self::chrono::NaiveDateTime;
use num_derive::{FromPrimitive, ToPrimitive};
use std::collections::HashMap;

use super::commit;
use super::event;
use super::file_item;
use super::resolvers;

/// Method how to perform commit of sieved images
#[derive(PartialEq, Eq, FromPrimitive, ToPrimitive, Clone, Debug)]
pub enum CommitMethod {
    /// Copy the images to be taken over to the target directory
    Copy = 0,
    /// Move the images to be taken over to the target directory
    Move,
    /// Move the images to be taken over to the target directory and delete the discarded files
    MoveAndDelete,
    /// Delete the discarded files
    Delete,
}

/// Item list containing all file items and all events
#[derive(Clone, Debug)]
pub struct ItemList {
    /// List of file items
    pub items: Vec<file_item::FileItem>,
    /// List of events
    pub events: Vec<event::Event>,
    /// Base path that was used to create the item list
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
            let item = Self::create_item(item_path, true, "");
            let path = item.get_path();
            if !self.items.iter().any(|i| i.get_path() == path) {
                self.items.push(item);
            }
        }
        self.items.sort();
        self.path = String::from(path);
    }

    /// Adds an item to the list
    pub fn add_item(&mut self, item_path: String, take_over: bool, encoded_hash: &str) {
        self.items
            .push(Self::create_item(item_path, take_over, encoded_hash));
    }

    /// Internal function to create a new file item
    fn create_item(item_path: String, take_over: bool, encoded_hash: &str) -> file_item::FileItem {
        let resolver = resolvers::get_resolver(&item_path);
        file_item::FileItem::new(item_path, resolver, take_over, encoded_hash)
    }

    /// Go through all images and find similar ones by comparing the timestamp
    pub fn find_similar(&mut self, max_diff_seconds: i64) {
        // Find similars based on the taken time
        if self.items.is_empty() {
            return;
        }
        let mut timestamp: i64 = 0;
        let mut start_similar_index: usize = 0;
        for index in 0..self.items.len() + 1 {
            if timestamp == 0 {
                timestamp = self.items[index].get_timestamp();
                start_similar_index = index;
            } else {
                if (index == self.items.len())
                    || (timestamp + max_diff_seconds < self.items[index].get_timestamp())
                {
                    // The item has a larger diff, so set all items between start_similar_index and index to be similar to each other
                    for similar_index in start_similar_index..index {
                        for other_index in start_similar_index..index {
                            if similar_index != other_index {
                                self.items[similar_index].add_similar(other_index);
                            }
                        }
                    }
                    start_similar_index = index;
                }
                if index < self.items.len() {
                    timestamp = self.items[index].get_timestamp();
                }
            }
        }
    }

    /// Go through all images and find similar ones by comparing the hash
    pub fn find_similar_hashes(&mut self, max_diff_hash: u32) {
        let mut similar_lists: HashMap<usize, Vec<(u32, usize)>> = HashMap::new();
        for index in 0..self.items.len() {
            similar_lists.insert(index, vec![]);
        }
        for index in 0..self.items.len() {
            for other_index in index + 1..self.items.len() {
                if other_index != index {
                    let distance = self.items[index].get_hash_distance(&self.items[other_index]);
                    if distance < max_diff_hash {
                        similar_lists
                            .get_mut(&index)
                            .unwrap()
                            .push((distance, other_index));
                        similar_lists
                            .get_mut(&other_index)
                            .unwrap()
                            .push((distance, index));
                    }
                }
            }
        }
        for index in 0..self.items.len() {
            let similar_list = similar_lists.get_mut(&index).unwrap();
            similar_list.sort_unstable();
            for (_, other_index) in similar_list {
                self.items[index].add_similar(*other_index);
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
        let commit_io = commit::FileCommitIO {};
        commit::commit(self, path, commit_method, &commit_io, progress_callback);
    }

    /// Gets the event which a file item belongs to
    pub fn get_event(&self, item: &file_item::FileItem) -> Option<&event::Event> {
        let naive_date = NaiveDateTime::from_timestamp(item.get_timestamp(), 0).date();
        for event in &self.events {
            if event.contains(&naive_date) {
                return Some(event);
            }
        }
        None
    }
}

/// Find all files in a directory with the extensions supported by FileItem
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
            let file = entry.expect("IO error");
            files.push(file.to_str().unwrap().to_string());
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
            #[allow(unsafe_code)]
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
                "",
            ));
        }
        let mut item_list = ItemList {
            items,
            events: vec![],
            path: String::from(""),
        };

        item_list.find_similar(5);

        assert_eq!(2, item_list.items[0].get_similars().len());
        assert_eq!(2, item_list.items[1].get_similars().len());
        assert_eq!(2, item_list.items[2].get_similars().len());
        assert_eq!(0, item_list.items[3].get_similars().len());
        assert_eq!(1, item_list.items[4].get_similars().len());
        assert_eq!(1, item_list.items[5].get_similars().len());
    }
}
