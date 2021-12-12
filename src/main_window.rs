//! Module containing the main window of image_sieve

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate rfd;
extern crate sixtyfps;

use num_traits::FromPrimitive;
use rfd::FileDialog;
use sixtyfps::{Model, SharedString};
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Mutex;
use std::thread;
use std::{cell::RefCell, sync::Arc};

use crate::item_sort_list::parse_date;
use crate::item_sort_list::{CommitMethod, ItemList};
use crate::misc::image_cache::{self, ImageCache, Purpose};
use crate::persistence::json::JsonPersistence;
use crate::persistence::json::{get_project_filename, get_settings_filename};
use crate::persistence::settings::Settings;
use crate::synchronize::Synchronizer;

#[allow(
    clippy::all,
    unused_qualifications,
    trivial_casts,
    trivial_numeric_casts,
    missing_docs,
    missing_debug_implementations
)]
mod generated_code {
    sixtyfps::include_modules!();
}
pub use generated_code::*;

type ImagesModelMap = HashMap<usize, usize>;

/// Main window container of the image sorter, contains the sixtyfps window, models and internal data structures
pub struct MainWindow {
    window: ImageSieve,
    item_list: Arc<Mutex<ItemList>>,
    item_list_model: Rc<sixtyfps::VecModel<SharedString>>,
    similar_items_model: Rc<sixtyfps::VecModel<SortImage>>,
    items_model_map: Rc<RefCell<ImagesModelMap>>,
    events_model: Rc<sixtyfps::VecModel<Event>>,
    commit_result_model: Rc<sixtyfps::VecModel<CommitResult>>,
    image_cache: Rc<ImageCache>,
    synchronizer: Rc<Synchronizer>,
}

impl Default for MainWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for MainWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let item_list = self.item_list.lock().unwrap();
        write!(f, "MainWindow").ok();
        write!(f, " item_list: {:?}", item_list).ok();
        Ok(())
    }
}

impl MainWindow {
    /// Creates a new main window and initializes it from saved settings
    pub fn new() -> Self {
        // Load settings and item list
        let settings: Settings =
            JsonPersistence::load(&get_settings_filename()).unwrap_or_else(Settings::new);

        let item_list = ItemList {
            items: vec![],
            events: vec![],
            path: PathBuf::new(),
        };

        let item_list = Arc::new(Mutex::new(item_list));

        let event_list_model = Rc::new(sixtyfps::VecModel::<Event>::default());
        let item_list_model = Rc::new(sixtyfps::VecModel::<SharedString>::default());
        let commit_result_model = Rc::new(sixtyfps::VecModel::<CommitResult>::default());

        // Construct main window
        let image_sieve = ImageSieve::new();

        let synchronizer = Synchronizer::new(item_list.clone(), &image_sieve);
        if !settings.source_directory.is_empty() {
            // Start synchronization in a background thread
            synchronizer.synchronize(
                Some(Path::new(&settings.source_directory)),
                settings.clone(),
            );
        }
        let mut cache = ImageCache::new();
        cache.restrict_size(1600, 1000);

        let main_window = Self {
            window: image_sieve,
            item_list,
            item_list_model,
            similar_items_model: Rc::new(sixtyfps::VecModel::<SortImage>::default()),
            items_model_map: Rc::new(RefCell::new(HashMap::new())),
            events_model: event_list_model,
            commit_result_model,
            image_cache: Rc::new(cache),
            synchronizer: Rc::new(synchronizer),
        };

        // Set initial values
        let version = env!("CARGO_PKG_VERSION");
        main_window
            .window
            .set_window_title(SharedString::from("ImageSieve v") + version);
        settings.to_window(&main_window.window);
        if settings.source_directory.is_empty() {
            main_window.window.set_loading(false);
            main_window.window.set_calculating_similarities(false);
        }

        // Set model references
        main_window
            .window
            .set_images_list_model(sixtyfps::ModelHandle::new(
                main_window.item_list_model.clone(),
            ));
        main_window
            .window
            .set_images_model(sixtyfps::ModelHandle::new(
                main_window.similar_items_model.clone(),
            ));
        main_window
            .window
            .set_events_model(sixtyfps::ModelHandle::new(main_window.events_model.clone()));
        main_window
            .window
            .set_commit_result_model(sixtyfps::ModelHandle::new(
                main_window.commit_result_model.clone(),
            ));

        main_window.setup_callbacks();

        main_window
    }

    /// Start the event loop
    pub fn run(&self) {
        self.window.run();

        self.synchronizer.stop();

        // Save settings when program exits
        let settings = Settings::from_window(&self.window);
        JsonPersistence::save(&get_settings_filename(), &settings);

        // and save item list
        let item_list = self.item_list.lock().unwrap();
        if !item_list.items.is_empty() || !item_list.events.is_empty() {
            JsonPersistence::save(&get_project_filename(&item_list.path), &item_list.clone());
        }
    }

    /// Setup sixtyfps GUI callbacks
    fn setup_callbacks(&self) {
        self.window.on_item_selected({
            // New item selected on the list of images or next/previous clicked
            let item_list = self.item_list.clone();
            let similar_items_model = self.similar_items_model.clone();
            let items_model_map = self.items_model_map.clone();
            let window_weak = self.window.as_weak();
            let image_cache = self.image_cache.clone();

            move |i: i32| {
                synchronize_images_model(
                    i as usize,
                    &item_list.lock().unwrap(),
                    similar_items_model.clone(),
                    &mut items_model_map.borrow_mut(),
                    window_weak.clone(),
                    &image_cache,
                );
            }
        });

        self.window.on_commit({
            // Commit pressed - perform selected action
            let window_weak = self.window.as_weak();
            let item_list = self.item_list.clone();
            let commit_result_model = self.commit_result_model.clone();

            move || {
                commit(
                    &item_list.lock().unwrap(),
                    window_weak.clone(),
                    commit_result_model.clone(),
                );
            }
        });

        self.window.on_take_over_toggle({
            // Image was clicked, toggle take over state
            let item_list_model = self.item_list_model.clone();
            let item_list = self.item_list.clone();
            let items_model = self.similar_items_model.clone();
            let items_model_map = self.items_model_map.clone();

            move |i: i32| {
                let model_index = i as usize;
                // Change the state of the SortImage in the items_model
                let mut sort_image = items_model.row_data(model_index);
                sort_image.take_over = !sort_image.take_over;

                let index = items_model_map.borrow_mut()[&model_index];
                {
                    // Change the item_list state
                    let mut item_list_mut = item_list.lock().unwrap();
                    item_list_mut.items[index].set_take_over(sort_image.take_over);
                }
                items_model.set_row_data(model_index, sort_image);
                // Update item list model to reflect change in icons in list
                synchronize_item_list_model(&item_list.lock().unwrap(), &item_list_model);
            }
        });

        self.window.on_browse_source({
            // Browse source was clicked, select new path
            let item_list_model = self.item_list_model.clone();
            let events_model = self.events_model.clone();
            let item_list = self.item_list.clone();
            let window_weak = self.window.as_weak();
            let synchronizer = self.synchronizer.clone();

            move || {
                let file_dialog = FileDialog::new();
                if let Some(folder) = file_dialog.pick_folder() {
                    {
                        // Save current item list
                        let item_list = item_list.lock().unwrap();
                        if !item_list.items.is_empty() {
                            JsonPersistence::save(
                                &get_project_filename(&item_list.path),
                                &item_list.clone(),
                            );
                        }
                    }

                    empty_model(item_list_model.clone());
                    empty_model(events_model.clone());

                    // Synchronize in a background thread
                    window_weak.unwrap().set_loading(true);
                    synchronizer
                        .synchronize(Some(&folder), Settings::from_window(&window_weak.unwrap()));

                    window_weak
                        .unwrap()
                        .set_source_directory(SharedString::from(folder.to_str().unwrap()));
                }
            }
        });

        self.window.on_browse_target({
            // Commit target path was changed
            let window_weak = self.window.as_weak();

            move || {
                let file_dialog = FileDialog::new();
                if let Some(folder) = file_dialog.pick_folder() {
                    let target_path = &String::from(folder.to_str().unwrap());

                    window_weak
                        .unwrap()
                        .set_target_directory(SharedString::from(target_path));
                }
            }
        });

        self.window.on_check_event({
            // Check event for overlapping dates
            let item_list = self.item_list.clone();

            move |start_date: SharedString,
                  end_date: SharedString,
                  new_event: bool|
                  -> SharedString {
                let start_date = parse_date(&start_date).unwrap();
                let end_date = parse_date(&end_date).unwrap();
                if start_date > end_date {
                    return SharedString::from("Start date must be before end date");
                }
                let item_list = item_list.lock().unwrap();
                let allowed_overlaps = if new_event { 0 } else { 1 };
                let mut overlaps = 0;
                for event in item_list.events.iter() {
                    if event.contains(&start_date) || event.contains(&end_date) {
                        overlaps += 1;
                        if overlaps > allowed_overlaps {
                            return SharedString::from(
                                String::from("Event overlaps with ") + &event.name,
                            );
                        }
                    }
                }
                SharedString::from("")
            }
        });

        self.window.on_add_event({
            // New event was added, return true if the dates are ok
            let item_list_model = self.item_list_model.clone();
            let events_model = self.events_model.clone();
            let item_list = self.item_list.clone();

            move |name, start_date: SharedString, end_date: SharedString| {
                let name_s = name.to_string();
                let event = crate::item_sort_list::Event::new(
                    name_s,
                    start_date.as_str(),
                    end_date.as_str(),
                )
                .unwrap();
                let mut item_list = item_list.lock().unwrap();
                item_list.events.push(event);
                item_list.events.sort_unstable();
                synchronize_event_list_model(&item_list, &events_model);
                // Synchronize the item list to update the icons of the entries
                synchronize_item_list_model(&item_list, &item_list_model.clone());
            }
        });

        self.window.on_date_valid(|date: SharedString| -> bool {
            crate::item_sort_list::Event::is_date_valid(date.to_string().as_str())
        });

        self.window.on_update_event({
            let item_list_model = self.item_list_model.clone();
            let events_model = self.events_model.clone();
            let item_list = self.item_list.clone();

            move |index| {
                let index = index as usize;
                let event = events_model.row_data(index);
                let mut item_list = item_list.lock().unwrap();
                if item_list.events[index].update(
                    event.name.to_string(),
                    event.start_date.as_str(),
                    event.end_date.as_str(),
                ) {
                    item_list.events.sort_unstable();
                    synchronize_event_list_model(&item_list, &events_model);
                    synchronize_item_list_model(&item_list, &item_list_model.clone());
                }
            }
        });

        self.window.on_remove_event({
            // Event was removed
            let item_list_model = self.item_list_model.clone();
            let events_model = self.events_model.clone();
            let item_list = self.item_list.clone();

            move |index| {
                events_model.remove(index as usize);
                let mut item_list = item_list.lock().unwrap();
                item_list.events.remove(index as usize);
                // Synchronize the item list to update the icons of the entries
                synchronize_item_list_model(&item_list, &item_list_model.clone());
            }
        });

        self.window.on_open({
            let item_list = self.item_list.clone();
            move |i: i32| {
                let item_list = item_list.lock().unwrap();
                let item = &item_list.items[i as usize];
                opener::open(&item.path).ok();
            }
        });

        self.window.on_open_url({
            move |url: SharedString| {
                opener::open(url.as_str()).ok();
            }
        });

        self.window.on_recheck_similarities({
            // Browse source was clicked, select new path
            let window_weak = self.window.as_weak();
            let synchronizer = self.synchronizer.clone();

            move || {
                // Synchronize in a background thread
                window_weak.unwrap().set_calculating_similarities(true);
                synchronizer.synchronize(None, Settings::from_window(&window_weak.unwrap()));
            }
        });

        self.window.on_cancel_loading({
            let synchronizer = self.synchronizer.clone();
            move || {
                synchronizer.stop();
            }
        });
    }
}

fn empty_model<T: 'static + Clone>(item_list_model: Rc<sixtyfps::VecModel<T>>) {
    for _ in 0..item_list_model.row_count() {
        item_list_model.remove(0);
    }
}

/// Synchronizes the list of found items from the internal data structure with the sixtyfps VecModel
pub fn synchronize_item_list_model(
    item_list: &ItemList,
    item_list_model: &sixtyfps::VecModel<SharedString>,
) {
    let empty_model = item_list_model.row_count() == 0;
    for (index, image) in item_list.items.iter().enumerate() {
        let mut item_string = image.get_item_string(&item_list.path);
        if item_list.get_event(image).is_some() {
            item_string = String::from("\u{1F4C5}") + &item_string;
        }
        if empty_model {
            item_list_model.push(SharedString::from(item_string));
        } else {
            item_list_model.set_row_data(index, SharedString::from(item_string));
        }
    }
}

/// Synchronize the event list with the GUI model
pub fn synchronize_event_list_model(
    item_list: &ItemList,
    event_list_model: &sixtyfps::VecModel<Event>,
) {
    let model_count = event_list_model.row_count();
    // Event model
    for (index, event) in item_list.events.iter().enumerate() {
        let _event = Event {
            name: SharedString::from(event.name.clone()),
            start_date: SharedString::from(event.start_date_as_string()),
            end_date: SharedString::from(event.end_date_as_string()),
        };
        if index >= model_count {
            event_list_model.push(_event);
        } else {
            event_list_model.set_row_data(index, _event);
        }
    }
}

/// Synchronizes the images to show at the same time from a selected image to the sixtyfps VecModel
fn synchronize_images_model(
    selected_item_index: usize,
    item_list: &ItemList,
    similar_items_model: Rc<sixtyfps::VecModel<SortImage>>,
    item_model_map: &mut ImagesModelMap,
    window: sixtyfps::Weak<ImageSieve>,
    image_cache: &ImageCache,
) {
    let similars = item_list.items[selected_item_index].get_similars();

    // Clear images model and the model map
    for _ in 0..similar_items_model.row_count() {
        similar_items_model.remove(0);
    }
    item_model_map.drain();

    let mut model_index: usize = 0;

    let mut add_item = |item_index: &usize,
                        selected_image: bool,
                        window_weak: sixtyfps::Weak<ImageSieve>| {
        let item = &item_list.items[*item_index];
        let image = {
            let image = image_cache.get(item);
            if let Some(image) = image {
                image
            } else {
                let f: image_cache::DoneCallback = Box::new(move |image_buffer| {
                    window_weak.clone().upgrade_in_event_loop(move |handle| {
                        if handle.get_current_list_item() == selected_item_index as i32 {
                            let mut row_data = handle.get_images_model().row_data(model_index);
                            let is_current_image = handle.get_current_image().text == row_data.text;
                            row_data.image = crate::misc::images::get_sixtyfps_image(&image_buffer);
                            handle
                                .get_images_model()
                                .set_row_data(model_index, row_data);
                            if is_current_image {
                                let mut current_image = handle.get_current_image();
                                current_image.image =
                                    crate::misc::images::get_sixtyfps_image(&image_buffer);
                                handle.set_current_image(current_image);
                            }
                        }
                    })
                });
                image_cache.load(
                    item,
                    if selected_image {
                        Purpose::SelectedImage
                    } else {
                        Purpose::SimilarImage
                    },
                    Some(f),
                );
                image_cache.get_waiting()
            }
        };

        let sort_image_struct = SortImage {
            image,
            take_over: item.get_take_over(),
            text: get_item_text(*item_index, item_list),
        };
        similar_items_model.push(sort_image_struct);
        item_model_map.insert(model_index, *item_index);
        model_index += 1;
    };

    // TODO: Also first item should be loaded in background, but in a prioritized queue
    add_item(&selected_item_index, true, window.clone());

    for image_index in similars {
        add_item(image_index, false, window.clone());
    }

    // Prefetch next two images
    let mut prefetch_index = selected_item_index + 1;
    let mut prefetches = 2;
    while prefetches > 0 && prefetch_index < item_list.items.len() {
        if !similars.contains(&prefetch_index) {
            if let Some(file_item) = item_list.items.get(prefetch_index) {
                if file_item.is_image() {
                    image_cache.load(file_item, Purpose::Prefetch, None);
                    prefetches -= 1;
                }
            }
        }
        prefetch_index += 1;
    }

    // Set properties
    window
        .unwrap()
        .set_current_image(similar_items_model.row_data(0));
    window.unwrap().set_current_image_index(0);
}

/// Gets the text of a an item at a given index as a SharedString
pub fn get_item_text(index: usize, item_list: &ItemList) -> SharedString {
    let item = &item_list.items[index];
    let event = item_list.get_event(item);
    let event_str = if let Some(event) = event {
        event.name.as_str()
    } else {
        ""
    };
    SharedString::from(item.to_string() + event_str)
}

/// Commits the item list in a background thread
pub fn commit(
    item_list: &ItemList,
    window_weak: sixtyfps::Weak<ImageSieve>,
    commit_result_model: Rc<sixtyfps::VecModel<CommitResult>>,
) {
    let item_list_copy = item_list.to_owned();
    let target_path = window_weak.unwrap().get_target_directory().to_string();
    let commit_method = FromPrimitive::from_i32(window_weak.unwrap().get_commit_method())
        .unwrap_or(CommitMethod::Copy);
    for _ in 0..commit_result_model.row_count() {
        commit_result_model.remove(0);
    }
    commit_result_model.push(CommitResult {
        result: SharedString::from(format!(
            "Committing using {:?} method to {}",
            commit_method, target_path
        )),
        color: SharedString::from("black"),
    });

    thread::spawn(move || {
        let progress_callback = |progress: String| {
            let window_weak_copy = window_weak.clone();
            window_weak_copy.upgrade_in_event_loop(move |handle| {
                if progress == "Done" {
                    handle.set_commit_running(false);
                }
                let commit_result_model = handle.get_commit_result_model();
                let commit_result_model = commit_result_model
                    .as_any()
                    .downcast_ref::<sixtyfps::VecModel<CommitResult>>()
                    .unwrap();
                let color = if progress == "Done" {
                    SharedString::from("green")
                } else if progress.starts_with("Error") {
                    SharedString::from("red")
                } else {
                    SharedString::from("black")
                };
                let commit_result = CommitResult {
                    result: SharedString::from(progress),
                    color,
                };
                commit_result_model.push(commit_result);
            });
        };
        item_list_copy.commit(Path::new(&target_path), commit_method, progress_callback);
    });
}
