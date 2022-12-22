use crate::item_sort_list::ItemList;
use crate::persistence::settings::Settings;
use image_23::GenericImageView;
use img_hash::HashAlg;
use img_hash::Hasher;
use img_hash::HasherConfig;
use img_hash::ImageHash;
use slint::ComponentHandle;
use slint::SharedString;
use walkdir::WalkDir;

use crate::main_window::ImageSieve;
use crate::persistence::json::get_project_filename;
use crate::persistence::json::JsonPersistence;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::sync::Mutex;

/// Combined path and settings used to send changes to the synchronize thread.
enum Command {
    Stop,
    Scan(PathBuf),
    Similarities(Settings),
}

/// Synchronize the item list with the state of the file system and calculate similarities in a background thread.
pub struct Synchronizer {
    channel: Sender<Command>,
}

impl Synchronizer {
    /// Creates a new synchronizer that is used to update the contents of an item list and
    /// set the resulting states in the ImageSieve window
    pub fn new(item_list: Arc<Mutex<ItemList>>, image_sieve: &ImageSieve) -> Self {
        let (channel, receiver) = mpsc::channel();
        std::thread::spawn({
            let handle_weak = image_sieve.as_weak();
            move || {
                synchronize_run(item_list, &receiver, handle_weak);
            }
        });
        Self { channel }
    }

    /// Perform synchronization of the item list with a given path in a background thread.
    pub fn scan_path(&self, path: &Path) {
        let path = path.to_path_buf();
        self.channel.send(Command::Scan(path)).ok();
    }

    /// Calculate similarities in a background thread.
    pub fn calculate_similarities(&self, settings: Settings) {
        self.channel.send(Command::Similarities(settings)).ok();
    }

    /// Stop the current synchronization process
    pub fn stop(&self) {
        self.channel.send(Command::Stop).ok();
    }
}

/// Dropping the object will cause the thread to exit by sending an empty path/settings command.
impl Drop for Synchronizer {
    fn drop(&mut self) {
        self.channel.send(Command::Stop).ok();
    }
}

/// Synchronization thread function
/// Receives a path and/or updated settings via a Receiver and processes this data. The given path is synchronized
/// with the currently loaded item list and the results are updated in the GUI. While the new item list is then already
/// shown, the computation intensive scanning for similarities is done in this thread as well. First, the timestamp
/// similarity is calculated and afterwards the image similarity (depending on if it is enabled or not)
fn synchronize_run(
    item_list: Arc<Mutex<ItemList>>,
    receiver: &Receiver<Command>,
    image_sieve: slint::Weak<ImageSieve>,
) {
    for command in receiver {
        // In any case, reset similarities first
        {
            let mut item_list_loc = item_list.lock().unwrap();
            for item in &mut item_list_loc.items {
                item.reset_similars();
            }
        }

        match command {
            Command::Stop => break,
            Command::Scan(path) => {
                if scan_files(&path, item_list.clone(), &image_sieve, receiver).is_err() {
                    let mut item_list_loc = item_list.lock().unwrap();
                    item_list_loc.items.clear();
                }
                image_sieve
                    .clone()
                    .upgrade_in_event_loop({
                        move |h| {
                            h.invoke_synchronization_finished();
                        }
                    })
                    .unwrap();
            }
            Command::Similarities(settings) => {
                // First, find similars based on times, this is usually quick
                if settings.settings_v05.use_timestamps {
                    calculate_similar_timestamps(item_list.clone(), &settings);
                }
                // Tell the GUI that this is done
                similarities_calculated(&image_sieve, !settings.settings_v05.use_hash);

                // Then, if enabled, find similars based on hashes. This takes some time.
                if settings.settings_v05.use_hash {
                    calculate_similar_hashes(item_list.clone(), &settings);
                    // Finally, update the GUI again with the new found similarities
                    similarities_calculated(&image_sieve, true);
                }
            }
        };
    }
}

/// Tell the GUI that the similarities have been calculated
fn similarities_calculated(image_sieve: &slint::Weak<ImageSieve>, finished: bool) {
    image_sieve
        .clone()
        .upgrade_in_event_loop({
            move |h| {
                h.invoke_similarities_calculated(finished);
            }
        })
        .unwrap();
}

/// Scan files in a path, update the item list with those found files and update the GUI models with the new data
fn scan_files(
    path: &Path,
    item_list: Arc<Mutex<ItemList>>,
    image_sieve: &slint::Weak<ImageSieve>,
    receiver: &Receiver<Command>,
) -> Result<(), ()> {
    let mut item_list_loc = item_list.lock().unwrap();

    item_list_loc.items.clear();

    report_progress(image_sieve, String::from("Checking existing project..."));
    check_abort(receiver)?;
    // Check if folder already contains an item list
    let loaded_item_list: Option<ItemList> = JsonPersistence::load(&get_project_filename(path));
    if let Some(loaded_item_list) = loaded_item_list {
        item_list_loc.clone_from(&loaded_item_list);
        item_list_loc.events.sort_unstable();
    }

    if !item_list_loc.items.is_empty() {
        report_progress(image_sieve, String::from("Checking existing files..."));
        check_abort(receiver)?;
        // First, drain missing files
        item_list_loc.drain_missing();
    }

    // Now, walk dirs and synchronize each
    for (file_counter, entry) in WalkDir::new(path).into_iter().flatten().enumerate() {
        if file_counter % 100 == 0 {
            report_progress(image_sieve, format!("Searching {}", entry.path().display()));
        }
        check_abort(receiver)?;
        item_list_loc.check_and_add(entry.path());
    }

    item_list_loc.finish_synchronizing(path);
    Ok(())
}

/// Check if an abort command was received
fn check_abort(receiver: &Receiver<Command>) -> Result<(), ()> {
    let command = receiver.try_recv();
    if let Ok(Command::Stop) = command {
        Err(())
    } else {
        Ok(())
    }
}

/// Extract the timestamp from all items in the item list and find similar items based on a maximum difference.
/// Afterwards, the GUI is updated with the new found similarities.
fn calculate_similar_timestamps(item_list: Arc<Mutex<ItemList>>, settings: &Settings) {
    {
        let mut item_list_loc = item_list.lock().unwrap();
        item_list_loc.find_similar(settings.settings_v05.timestamp_max_diff);
    }
}

/// Calculate the similarity hashes of images in the item list and check for hashes with a given maximum distance. Does not update the GUI
fn calculate_similar_hashes(item_list: Arc<Mutex<ItemList>>, settings: &Settings) {
    // Collect file names which need to be hashed (those that are images and have no stored hash yet)
    let mut image_file_names: Vec<PathBuf> = Vec::new();
    {
        let item_list_loc = item_list.lock().unwrap();
        for item in &item_list_loc.items {
            if (item.is_image() || item.is_raw_image()) && !item.has_hash() {
                image_file_names.push(item.path.clone());
            }
        }
    }

    // Now calculate the hashes
    let mut hashes: HashMap<PathBuf, ImageHash<Vec<u8>>> = HashMap::new();
    for image_file_name in image_file_names {
        if let Ok(image) = image_23::open(&image_file_name) {
            // The hash size is dependent on the image orientation to increase the result quality
            let (hash_width, hash_height) = if image.width() > image.height() {
                (16, 8)
            } else {
                (8, 16)
            };
            // We are using the double gradient algorithm
            let hasher: Hasher<Vec<u8>> = HasherConfig::with_bytes_type()
                .hash_size(hash_width, hash_height)
                .hash_alg(HashAlg::DoubleGradient)
                .to_hasher();
            hashes.insert(image_file_name, hasher.hash_image(&image));
        }
    }

    // Update the items with the new calculated hashes and update the similarities
    {
        let mut item_list_loc = item_list.lock().unwrap();
        for item in &mut item_list_loc.items {
            let hash = hashes.remove(&item.path);
            if let Some(hash) = hash {
                item.set_hash(hash);
            }
        }
        item_list_loc.find_similar_hashes(settings.settings_v05.hash_max_diff);
    }
}

/// Report a progress string back to the main window
fn report_progress(image_sieve: &slint::Weak<ImageSieve>, progress: String) {
    image_sieve
        .clone()
        .upgrade_in_event_loop({
            move |h| {
                h.set_loading_progress(SharedString::from(progress));
            }
        })
        .unwrap();
}
