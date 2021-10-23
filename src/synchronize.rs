use crate::item_sort_list::ItemList;
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
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

pub struct Synchronizer {
    channel: Sender<String>,
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

    /// Perform synchronization of the item list with a given path in a background thread
    pub fn synchronize(&self, path: &str) {
        self.channel.send(String::from(path)).ok();
    }
}

/// Synchronization thread function
fn synchronize_run(
    item_list: Arc<Mutex<ItemList>>,
    receiver: Receiver<String>,
    image_sieve: sixtyfps::Weak<ImageSieve>,
) {
    for path in receiver {
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
        thread::sleep(Duration::from_millis(1000));
        {
            let mut item_list_loc = item_list.lock().unwrap();

            item_list_loc.find_similar(5, 20);
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
            }
        });
    }
}
