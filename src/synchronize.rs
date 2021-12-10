use crate::item_sort_list::ItemList;
use crate::persistence::settings::Settings;
use image::GenericImageView;
use img_hash::HashAlg;
use img_hash::Hasher;
use img_hash::HasherConfig;
use img_hash::ImageHash;
use sixtyfps::ComponentHandle;
use sixtyfps::Model;
use sixtyfps::SharedString;
use sixtyfps::VecModel;
use walkdir::WalkDir;

use crate::main_window::synchronize_event_list_model;
use crate::main_window::synchronize_item_list_model;
use crate::main_window::Event;
use crate::main_window::ImageSieve;
use crate::misc::images::get_empty_image;
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
    Scan(PathBuf, Settings),
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
                synchronize_run(item_list, receiver, handle_weak);
            }
        });
        Self { channel }
    }

    /// Perform synchronization of the item list with a given path in a background thread. If the path is
    /// a zero length string, only check for similarities
    pub fn synchronize(&self, path: Option<&Path>, settings: Settings) {
        if let Some(path) = path {
            let path = path.to_path_buf();
            self.channel.send(Command::Scan(path, settings)).ok();
        } else {
            self.channel.send(Command::Similarities(settings)).ok();
        }
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
    receiver: Receiver<Command>,
    image_sieve: sixtyfps::Weak<ImageSieve>,
) {
    for command in receiver {
        // In any case, reset similarities first
        {
            let mut item_list_loc = item_list.lock().unwrap();
            for item in &mut item_list_loc.items {
                item.reset_similars();
            }
        }

        let settings = match command {
            Command::Stop => break,
            Command::Scan(path, settings) => {
                scan_files(&path, item_list.clone(), &image_sieve);
                settings
            }
            Command::Similarities(settings) => settings,
        };

        // First, find similars based on times, this is usually quick
        if settings.use_timestamps {
            calculate_similar_timestamps(item_list.clone(), &image_sieve, &settings);
        }

        // Then, if enabled, find similars based on hashes. This takes some time.
        if settings.use_hash {
            calculate_similar_hashes(item_list.clone(), &settings);
        }

        // Finally, update the GUI again with the new found similarities
        image_sieve.clone().upgrade_in_event_loop({
            let item_list = item_list.lock().unwrap().to_owned();
            move |h| {
                synchronize_item_list_model(
                    &item_list,
                    h.get_images_list_model()
                        .as_any()
                        .downcast_ref::<VecModel<SharedString>>()
                        .unwrap(),
                );
                h.set_calculating_similarities(false);
            }
        });
    }
}

/// Scan files in a path, update the item list with those found files and update the GUI models with the new data
fn scan_files(
    path: &Path,
    item_list: Arc<Mutex<ItemList>>,
    image_sieve: &sixtyfps::Weak<ImageSieve>,
) {
    {
        let mut item_list_loc = item_list.lock().unwrap();

        // Check if folder already contains an item list
        let loaded_item_list: Option<ItemList> = JsonPersistence::load(&get_project_filename(path));
        if let Some(loaded_item_list) = loaded_item_list {
            item_list_loc.clone_from(&loaded_item_list);
        }

        // First, drain missing files
        item_list_loc.drain_missing();

        // Now, walk dirs and synchronize each
        for entry in WalkDir::new(path).into_iter().flatten() {
            item_list_loc.synchronize(entry.path());
        }

        item_list_loc.finish_synchronizing(path);
    }
    image_sieve.clone().upgrade_in_event_loop({
        let item_list = item_list.lock().unwrap().to_owned();
        move |h| {
            synchronize_item_list_model(
                &item_list,
                h.get_images_list_model()
                    .as_any()
                    .downcast_ref::<VecModel<SharedString>>()
                    .unwrap(),
            );
            synchronize_event_list_model(
                &item_list,
                h.get_events_model()
                    .as_any()
                    .downcast_ref::<VecModel<Event>>()
                    .unwrap(),
            );

            // Update the selection variables
            let num_items = { item_list.items.len() as i32 };
            if num_items > 0 {
                h.invoke_item_selected(0);
            } else {
                let empty_image = crate::main_window::SortImage {
                    image: get_empty_image(),
                    take_over: true,
                    text: SharedString::from("No images found"),
                };
                h.set_current_image(empty_image);
                h.set_current_image_index(0);
            }

            // And finally enable the GUI by setting the loading flag to false
            h.set_loading(false);
        }
    });
}

/// Extract the timestamp from all items in the item list and find similar items based on a maximum difference.
/// Afterwards, the GUI is updated with the new found similarities.
fn calculate_similar_timestamps(
    item_list: Arc<Mutex<ItemList>>,
    image_sieve: &sixtyfps::Weak<ImageSieve>,
    settings: &Settings,
) {
    {
        let mut item_list_loc = item_list.lock().unwrap();
        item_list_loc.find_similar(settings.timestamp_max_diff);
    }

    image_sieve.clone().upgrade_in_event_loop({
        let item_list = item_list.lock().unwrap().to_owned();
        let use_hash = settings.use_hash;
        move |h| {
            synchronize_item_list_model(
                &item_list,
                h.get_images_list_model()
                    .as_any()
                    .downcast_ref::<VecModel<SharedString>>()
                    .unwrap(),
            );
            h.set_calculating_similarities(use_hash);
        }
    });
}

/// Calculate the similarity hashes of images in the item list and check for hashes with a given maximum distance. Does not update the GUI
fn calculate_similar_hashes(item_list: Arc<Mutex<ItemList>>, settings: &Settings) {
    // Collect file names which need to be hashed (those that are images and have no stored hash yet)
    let mut image_file_names: Vec<PathBuf> = Vec::new();
    {
        let item_list_loc = item_list.lock().unwrap();
        for item in &item_list_loc.items {
            if item.is_image() && !item.has_hash() {
                image_file_names.push(item.path.clone());
            }
        }
    }

    // Now calculate the hashes
    let mut hashes: HashMap<PathBuf, ImageHash<Vec<u8>>> = HashMap::new();
    for image_file_name in image_file_names {
        if let Ok(image) = image::open(&image_file_name) {
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
        item_list_loc.find_similar_hashes(settings.hash_max_diff);
    }
}
