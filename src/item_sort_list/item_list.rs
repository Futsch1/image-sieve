extern crate chrono;

use chrono::DateTime;
use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use super::event;
use super::file_item;
use super::file_types::is_any;
use super::resolvers;
use super::sieve;

/// Method how to perform sieve of sieved images
#[derive(PartialEq, Eq, FromPrimitive, ToPrimitive, Clone, Debug, Serialize, Deserialize)]
#[repr(i32)]
pub enum SieveMethod {
    /// Copy the images to be taken over to the target directory
    Copy = 0,
    /// Move the images to be taken over to the target directory
    Move,
    /// Move the images to be taken over to the target directory and delete the discarded files
    MoveAndDelete,
    /// Delete the discarded files
    Delete,
}

#[derive(PartialEq, Eq, FromPrimitive, ToPrimitive, Clone, Debug, Serialize, Deserialize)]
#[repr(i32)]
pub enum DirectoryNames {
    /// Directories are named by year and month
    YearAndMonth = 0,
    /// Directories are named by year
    Year,
    /// Directories are named by year, month and day
    YearMonthAndDay,
    /// Directories are named by year and quarter
    YearAndQuarter,
    /// Directories are named by year and subdirectory by month
    YearAndMonthInSubdirectory,
}

/// Item list containing all file items and all events
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemList {
    /// List of file items
    pub items: Vec<file_item::FileItem>,
    /// List of events
    pub events: Vec<event::Event>,
    /// Base path that was used to create the item list
    pub path: PathBuf,
}

impl Default for ItemList {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemList {
    pub fn new() -> Self {
        resolvers::init_resolvers();
        ItemList {
            items: vec![],
            events: vec![],
            path: PathBuf::new(),
        }
    }

    /// Remove all missing files from the item list
    pub fn drain_missing(&mut self) {
        self.items = self.items.drain(..).filter(|i| i.path.exists()).collect();
    }

    /// Check if a path can be added
    pub fn check_and_add(&mut self, path: &Path) {
        if is_any(path) && !self.items.iter().any(|i| i.path == path) {
            let item = Self::create_item(path.to_path_buf(), true, "");
            self.items.push(item);
        }
    }

    /// Returns the index of a file item
    pub fn index_of_item(&self, item: &file_item::FileItem) -> Option<usize> {
        self.items.iter().position(|i| i.path == item.path)
    }

    /// Returns the index of a file item by its path
    pub fn index_of_path(&self, path: &str) -> Option<usize> {
        self.items.iter().position(|i| i.path.to_str().unwrap_or("") == path)
    }

    /// Finish the synchronization progress
    pub fn finish_synchronizing(&mut self, base_path: &Path) {
        self.items.sort();
        self.path = base_path.to_path_buf();
    }

    /// Adds an item to the list
    pub fn add_item(&mut self, item_path: &Path, take_over: bool, encoded_hash: &str) {
        self.items.push(Self::create_item(
            item_path.to_path_buf(),
            take_over,
            encoded_hash,
        ));
    }

    /// Internal function to create a new file item
    fn create_item(item_path: PathBuf, take_over: bool, encoded_hash: &str) -> file_item::FileItem {
        let resolver = resolvers::get_resolver(&item_path);
        file_item::FileItem::new(item_path, resolver, take_over, encoded_hash)
    }

    /// Go through all images and find similar ones by comparing the timestamp
    pub fn find_similar(&mut self, max_diff_seconds: i64) {
        // Find similars based on the taken time
        if self.items.is_empty() {
            return;
        }
        let mut timestamp: i64 = self.items[0].get_timestamp();
        let mut start_similar_index: usize = 0;
        for index in 0..self.items.len() {
            if timestamp + max_diff_seconds < self.items[index].get_timestamp() {
                // The item has a larger diff, so set all items between start_similar_index and index to be similar to each other
                self.set_similar_range(start_similar_index..index);

                start_similar_index = index;
            }
            timestamp = self.items[index].get_timestamp();
        }
        // Set all the remaining indices
        self.set_similar_range(start_similar_index..self.items.len());
        // Now remove the own index from all items
        for index in 0..self.items.len() {
            self.items[index].clean_similars(index);
        }
    }

    /// Sets a range of similar indices for all items in that range
    fn set_similar_range(&mut self, index_range: std::ops::Range<usize>) {
        for similar_index in index_range.clone() {
            self.items[similar_index].add_similar_range(&index_range);
        }
    }

    /// Go through all images and find similar ones by comparing the hash
    pub fn find_similar_hashes(&mut self, max_diff_hash: u32) {
        let mut similar_lists: HashMap<usize, Vec<usize>> = HashMap::new();
        for index in 0..self.items.len() {
            similar_lists.insert(index, vec![]);
        }
        for index in 0..self.items.len() {
            for other_index in index + 1..self.items.len() {
                if other_index != index {
                    let distance = self.items[index].get_hash_distance(&self.items[other_index]);
                    if distance < max_diff_hash {
                        similar_lists.get_mut(&index).unwrap().push(other_index);
                        similar_lists.get_mut(&other_index).unwrap().push(index);
                    }
                }
            }
        }
        for index in 0..self.items.len() {
            let similar_list = similar_lists.get(&index).unwrap();
            self.items[index].add_similar_vec(similar_list);
            self.items[index].clean_similars(index);
        }
    }

    /// Sieves an item list taking the take_over flag into account to a new directory.
    /// The progress is reported by calling a callback function with the file that is currently processed.
    pub fn sieve(
        &self,
        path: &Path,
        sieve_method: SieveMethod,
        sieve_directory_names: DirectoryNames,
        progress_callback: impl Fn(String),
    ) {
        let sieve_io = sieve::FileSieveIO {};
        sieve::sieve(
            self,
            path,
            sieve_method,
            sieve_directory_names,
            &sieve_io,
            progress_callback,
        );
    }

    /// Gets the event which a file item belongs to
    pub fn get_event(&self, item: &file_item::FileItem) -> Option<&event::Event> {
        let naive_date = DateTime::from_timestamp(item.get_timestamp(), 0)
            .unwrap().date_naive();
        self.events
            .iter()
            .find(|&event| event.contains(&naive_date))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item_sort_list::item_traits::PropertyResolver;
    use std::cell::RefCell;
    use std::rc::Rc;
    extern crate base64;
    use base64::{engine::general_purpose, Engine as _};

    struct MockResolver {
        call_count: Rc<RefCell<usize>>,
    }

    impl MockResolver {
        pub fn new(call_count: Rc<RefCell<usize>>) -> Self {
            MockResolver { call_count }
        }
    }

    impl PropertyResolver for MockResolver {
        fn get_timestamp(&self) -> i64 {
            let return_values: [i64; 6] = [0, 4, 8, 14, 64, 65];
            let call_count = *self.call_count.borrow_mut();
            self.call_count.replace(call_count + 1);
            return_values[call_count]
        }

        fn get_orientation(&self) -> Option<crate::item_sort_list::Orientation> {
            None
        }
    }

    #[test]
    fn find_similar() {
        let call_count = Rc::new(RefCell::new(0));

        let mut items: Vec<file_item::FileItem> = vec![];
        for _ in 0..6 {
            items.push(file_item::FileItem::new(
                PathBuf::from("test.jpg"),
                Box::new(MockResolver::new(call_count.clone())),
                true,
                "",
            ));
        }
        let mut item_list = ItemList {
            items,
            events: vec![],
            path: PathBuf::from(""),
        };

        item_list.find_similar(5);

        assert_eq!(2, item_list.items[0].get_similars().len());
        assert_eq!(2, item_list.items[1].get_similars().len());
        assert_eq!(2, item_list.items[2].get_similars().len());
        assert_eq!(0, item_list.items[3].get_similars().len());
        assert_eq!(1, item_list.items[4].get_similars().len());
        assert_eq!(1, item_list.items[5].get_similars().len());
    }

    #[test]
    fn find_similar_hashes() {
        let call_count = Rc::new(RefCell::new(0));

        let mut items: Vec<file_item::FileItem> = vec![];
        let hashes = ["a", "b", "c", "h", "i", "j"];
        for hash in hashes {
            let encoded = general_purpose::STANDARD.encode(hash);
            items.push(file_item::FileItem::new(
                PathBuf::from("test.jpg"),
                Box::new(MockResolver::new(call_count.clone())),
                true,
                &encoded,
            ));
        }
        let mut item_list = ItemList {
            items,
            events: vec![],
            path: PathBuf::from(""),
        };

        item_list.find_similar_hashes(2);

        assert_eq!(2, item_list.items[0].get_similars().len());
        assert_eq!(2, item_list.items[4].get_similars().len());
    }

    #[test]
    fn updating() {
        let mut item_list = ItemList {
            items: vec![],
            events: vec![],
            path: PathBuf::from(""),
        };

        item_list.check_and_add(Path::new("tests/test_no_date.jpg"));
        item_list.check_and_add(Path::new("tests/test_no_exif.jpg"));
        item_list.check_and_add(Path::new("tests/test.jpg"));
        item_list.check_and_add(Path::new("tests/test_no_date.jpg"));
        item_list.check_and_add(Path::new("tests/test_invalid.jpg"));
        item_list.check_and_add(Path::new("tests/test"));
        assert_eq!(4, item_list.items.len());
        assert_eq!(Some(0), item_list.index_of_path("tests/test_no_date.jpg"));

        item_list.finish_synchronizing(Path::new("tests"));
        assert_eq!("tests", item_list.path.to_str().unwrap());

        item_list.add_item(Path::new("tests/not_there.jpg"), true, "");
        assert_eq!(5, item_list.items.len());
        item_list.drain_missing();
        assert_eq!(4, item_list.items.len());
    }
}
