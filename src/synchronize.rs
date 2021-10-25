use crate::item_sort_list::ItemList;
use crate::persistence::settings::Settings;
use img_hash::Hasher;
use img_hash::HasherConfig;
use img_hash::ImageHash;
use sixtyfps::ComponentHandle;
use sixtyfps::Model;
use sixtyfps::SharedString;
use sixtyfps::VecModel;

use crate::main_window::synchronize_event_list_model;
use crate::main_window::synchronize_item_list_model;
use crate::main_window::Event;
use crate::main_window::ImageSieve;
use crate::misc::images::get_empty_image;
use crate::persistence::json::get_project_filename;
use crate::persistence::json::JsonPersistence;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::sync::Mutex;

struct PathAndSettings {
    pub path: Option<String>,
    pub settings: Settings,
}

pub struct Synchronizer {
    channel: Sender<PathAndSettings>,
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
    pub fn synchronize(&self, path: &str, settings: Settings) {
        let path = String::from(path);
        let path = if !path.is_empty() { Some(path) } else { None };
        self.channel.send(PathAndSettings { path, settings }).ok();
    }
}

/// Synchronization thread function
fn synchronize_run(
    item_list: Arc<Mutex<ItemList>>,
    receiver: Receiver<PathAndSettings>,
    image_sieve: sixtyfps::Weak<ImageSieve>,
) {
    for path_and_settings in receiver {
        let path = path_and_settings.path;
        let settings = path_and_settings.settings;
        if let Some(path) = path {
            {
                let mut item_list_loc = item_list.lock().unwrap();

                // Check if folder already contains an item list
                let loaded_item_list: Option<ItemList> =
                    JsonPersistence::load(&get_project_filename(&path));
                if let Some(loaded_item_list) = loaded_item_list {
                    item_list_loc.clone_from(&loaded_item_list);
                }

                item_list_loc.synchronize(&path);
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
                    let num_items = { item_list.items.len() as i32 };
                    h.set_num_list_items(num_items);

                    if num_items > 0 {
                        h.invoke_item_selected(0);
                    } else {
                        let empty_image = crate::main_window::SortImage {
                            image: get_empty_image(),
                            take_over: true,
                        };
                        h.set_current_image(empty_image);
                        h.set_current_image_index(0);
                        h.set_num_images(0);
                        h.set_current_image_text(sixtyfps::SharedString::from("No images found"));
                    }

                    h.set_loading(false);
                }
            });
        }

        // In any case, reset similarities first
        {
            let mut item_list_loc = item_list.lock().unwrap();
            for item in &mut item_list_loc.items {
                item.reset_similars();
            }
        }

        // First, find similars based on times, this is usually quick
        if settings.use_timestamps {
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

        if settings.use_hash {
            // Calculate hashes
            let mut image_paths: Vec<String> = Vec::new();
            {
                let item_list_loc = item_list.lock().unwrap();
                for item in &item_list_loc.items {
                    if item.is_image() && !item.has_hash() {
                        image_paths.push(item.get_path_as_str().clone());
                    }
                }
            }

            let mut hashes: HashMap<String, ImageHash<Vec<u8>>> = HashMap::new();
            for image_path in image_paths {
                let image = image::open(&image_path).unwrap();
                let hasher: Hasher<Vec<u8>> = HasherConfig::with_bytes_type().to_hasher();
                hashes.insert(image_path, hasher.hash_image(&image));
            }

            {
                let mut item_list_loc = item_list.lock().unwrap();
                for item in &mut item_list_loc.items {
                    let hash = hashes.remove(item.get_path_as_str());
                    if hash.is_some() {
                        item.set_hash(hash);
                    }
                }
                item_list_loc.find_similar_hashes(settings.hash_max_diff);
            }
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
                h.set_calculating_similarities(false);
            }
        });
    }
}
