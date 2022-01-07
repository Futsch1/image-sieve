//! Module containing the main window of image_sieve

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate nfd;
extern crate sixtyfps;

use sixtyfps::{Model, ModelHandle, SharedString};
use std::fmt::Debug;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use crate::item_sort_list::ItemList;
use crate::misc::image_cache::{self, ImageCache, Purpose};
use crate::models::events_model;
use crate::models::gui_items::sort_item_from_file_item;
use crate::models::list_model::populate_list_model;
use crate::models::list_model::update_list_model;
use crate::persistence::json::JsonPersistence;
use crate::persistence::json::{get_project_filename, get_settings_filename};
use crate::persistence::model_to_enum::model_to_enum;
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

/// Main window container of the image sorter, contains the sixtyfps window, models and internal data structures
pub struct MainWindow {
    window: ImageSieve,
    item_list: Arc<Mutex<ItemList>>,
    list_model: Rc<sixtyfps::VecModel<ListItem>>,
    similar_images_model: Rc<sixtyfps::VecModel<SortItem>>,
    events_model: Rc<sixtyfps::VecModel<Event>>,
    sieve_result_model: Rc<sixtyfps::VecModel<SieveResult>>,
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

        let item_list = ItemList::new();

        let item_list = Arc::new(Mutex::new(item_list));

        let event_list_model = Rc::new(sixtyfps::VecModel::<Event>::default());
        let item_list_model = Rc::new(sixtyfps::VecModel::<ListItem>::default());
        let sieve_result_model = Rc::new(sixtyfps::VecModel::<SieveResult>::default());

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
            list_model: item_list_model,
            similar_images_model: Rc::new(sixtyfps::VecModel::<SortItem>::default()),
            events_model: event_list_model,
            sieve_result_model,
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
            .set_list_model(sixtyfps::ModelHandle::new(main_window.list_model.clone()));
        main_window
            .window
            .set_similar_images_model(sixtyfps::ModelHandle::new(
                main_window.similar_images_model.clone(),
            ));
        main_window
            .window
            .set_events_model(sixtyfps::ModelHandle::new(main_window.events_model.clone()));
        main_window
            .window
            .set_sieve_result_model(sixtyfps::ModelHandle::new(
                main_window.sieve_result_model.clone(),
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
            let list_model = self.list_model.clone();
            let similar_items_model = self.similar_images_model.clone();
            let window_weak = self.window.as_weak();
            let image_cache = self.image_cache.clone();

            move |i: i32| {
                let index = list_model.row_data(i as usize).local_index as usize;
                let item_list = &item_list.lock().unwrap();
                synchronize_similar_items_model(
                    index,
                    item_list,
                    similar_items_model.clone(),
                    window_weak.clone(),
                    &image_cache,
                );
                prefetch_images(i, item_list, &list_model, &image_cache);
            }
        });

        self.window.on_sieve({
            // Sieve pressed - perform selected action
            let window_weak = self.window.as_weak();
            let item_list = self.item_list.clone();
            let sieve_result_model = self.sieve_result_model.clone();

            move || {
                sieve(
                    &item_list.lock().unwrap(),
                    window_weak.clone(),
                    sieve_result_model.clone(),
                );
            }
        });

        self.window.on_set_take_over({
            // Image was clicked, toggle take over state
            let list_model = self.list_model.clone();
            let similar_items_model = self.similar_images_model.clone();
            let item_list = self.item_list.clone();

            move |i: i32, take_over: bool| {
                // Change the state of the SortImage in the items_model

                let index = i as usize;
                {
                    // Change the item_list state
                    let mut item_list_mut = item_list.lock().unwrap();
                    item_list_mut.items[index].set_take_over(take_over);
                }
                // Update item list model to reflect change in icons in list
                update_list_model(&item_list.lock().unwrap(), &list_model);
                // And update the take over state in the similar items model
                for count in 0..similar_items_model.row_count() {
                    let mut item: SortItem = similar_items_model.row_data(count);
                    if item.local_index == i {
                        item.take_over = take_over;
                        similar_items_model.set_row_data(count, item);
                        break;
                    }
                }
            }
        });

        self.window.on_browse_source({
            // Browse source was clicked, select new path
            let list_model = self.list_model.clone();
            let events_model = self.events_model.clone();
            let item_list = self.item_list.clone();
            let window_weak = self.window.as_weak();
            let synchronizer = self.synchronizer.clone();

            move || {
                if let Ok(nfd::Response::Okay(folder)) = nfd::open_pick_folder(Some(
                    window_weak
                        .unwrap()
                        .get_source_directory()
                        .to_string()
                        .as_str(),
                )) {
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

                    empty_model(list_model.clone());
                    empty_model(events_model.clone());

                    // Synchronize in a background thread
                    window_weak.unwrap().set_loading(true);
                    synchronizer.synchronize(
                        Some(Path::new(&folder)),
                        Settings::from_window(&window_weak.unwrap()),
                    );

                    window_weak
                        .unwrap()
                        .set_source_directory(SharedString::from(folder));
                }
            }
        });

        self.window.on_browse_target({
            // Sieve target path was changed
            let window_weak = self.window.as_weak();

            move || {
                if let Ok(nfd::Response::Okay(folder)) = nfd::open_pick_folder(Some(
                    window_weak
                        .unwrap()
                        .get_target_directory()
                        .to_string()
                        .as_str(),
                )) {
                    window_weak
                        .unwrap()
                        .set_target_directory(SharedString::from(folder));
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
                let item_list = item_list.lock().unwrap();
                events_model::check_event(&start_date, &end_date, new_event, &item_list)
            }
        });

        self.window.on_add_event({
            // New event was added, return true if the dates are ok
            let list_model = self.list_model.clone();
            let events_model = self.events_model.clone();
            let item_list = self.item_list.clone();

            move |name: SharedString, start_date: SharedString, end_date: SharedString| {
                let mut item_list = item_list.lock().unwrap();
                if events_model::add_event(
                    &name,
                    &start_date,
                    &end_date,
                    &mut item_list,
                    &events_model,
                ) {
                    update_list_model(&item_list, &list_model.clone());
                }
            }
        });

        self.window.on_date_valid(|date: SharedString| -> bool {
            crate::item_sort_list::Event::is_date_valid(date.to_string().as_str())
        });

        self.window.on_update_event({
            let list_model = self.list_model.clone();
            let events_model = self.events_model.clone();
            let item_list = self.item_list.clone();

            move |index| {
                let mut item_list = item_list.lock().unwrap();
                if events_model::update_event(index, &mut item_list, &events_model) {
                    update_list_model(&item_list, &list_model);
                }
            }
        });

        self.window.on_remove_event({
            // Event was removed
            let list_model = self.list_model.clone();
            let events_model = self.events_model.clone();
            let item_list = self.item_list.clone();

            move |index| {
                let mut item_list = item_list.lock().unwrap();
                events_model::remove_event(index, &mut item_list, &events_model);
                update_list_model(&item_list, &list_model);
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

        self.window.on_filter({
            let list_model = self.list_model.clone();
            let item_list = self.item_list.clone();
            let window_weak = self.window.as_weak();

            move |filters| {
                let item_list = item_list.lock().unwrap();
                empty_model(list_model.clone());
                populate_list_model(&item_list, &list_model, &filters);
                let rows = list_model.row_count() as i32;

                if rows >= window_weak.unwrap().get_current_list_item() {
                    window_weak.unwrap().set_current_list_item(rows - 1);
                }
            }
        });
    }
}

/// Removes all items from a model
fn empty_model<T: 'static + Clone>(vec_model: Rc<sixtyfps::VecModel<T>>) {
    for _ in 0..vec_model.row_count() {
        vec_model.remove(0);
    }
}

/// Synchronizes the images to show at the same time from a selected image to the sixtyfps VecModel
fn synchronize_similar_items_model(
    current_item_local_index: usize,
    item_list: &ItemList,
    similar_items_model: Rc<sixtyfps::VecModel<SortItem>>,
    window: sixtyfps::Weak<ImageSieve>,
    image_cache: &ImageCache,
) {
    let similars = item_list.items[current_item_local_index].get_similars();

    // Clear images model
    empty_model(similar_items_model.clone());

    let mut model_index: usize = 0;

    let mut add_item = |item_index: &usize,
                        current_image: bool,
                        has_similars: bool,
                        window_weak: sixtyfps::Weak<ImageSieve>| {
        let item = &item_list.items[*item_index];
        let image = {
            let image = image_cache.get(item);
            if let Some(image) = image {
                image
            } else {
                let f: image_cache::DoneCallback = Box::new(move |image_buffer| {
                    window_weak.clone().upgrade_in_event_loop(move |handle| {
                        // Check if still the image is visible that caused the image loads
                        if handle.get_current_image().local_index == current_item_local_index as i32
                        {
                            let mut row_data =
                                handle.get_similar_images_model().row_data(model_index);
                            if has_similars {
                                row_data.image =
                                    crate::misc::images::get_sixtyfps_image(&image_buffer);
                                handle
                                    .get_similar_images_model()
                                    .set_row_data(model_index, row_data);
                            }
                            // If the image is the current image, then we need to also update the current image SortImage
                            if current_image {
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
                    if current_image {
                        Purpose::CurrentImage
                    } else {
                        Purpose::SimilarImage
                    },
                    Some(f),
                );
                image_cache.get_waiting()
            }
        };

        let sort_image = sort_item_from_file_item(item, item_list, image);
        similar_items_model.push(sort_image);
        model_index += 1;
    };

    // Clear pending commands in the image cache
    image_cache.purge();

    add_item(
        &current_item_local_index,
        true,
        !similars.is_empty(),
        window.clone(),
    );
    for image_index in similars {
        add_item(image_index, false, !similars.is_empty(), window.clone());
    }

    // Set properties
    window
        .unwrap()
        .set_current_image(similar_items_model.row_data(0));
}

/// Prefetch the next images in the model list
fn prefetch_images(
    model_index: i32,
    item_list: &ItemList,
    list_model: &sixtyfps::VecModel<ListItem>,
    image_cache: &ImageCache,
) {
    // Prefetch next two images
    for i in model_index + 1..model_index + 3 {
        if i < list_model.row_count() as i32 {
            let list_item = &list_model.row_data(i as usize);
            let file_item = &item_list.items[list_item.local_index as usize];
            if file_item.is_image() {
                image_cache.load(file_item, Purpose::Prefetch, None);
            }
        }
    }
}

/// Sieves the item list in a background thread
pub fn sieve(
    item_list: &ItemList,
    window_weak: sixtyfps::Weak<ImageSieve>,
    sieve_result_model: Rc<sixtyfps::VecModel<SieveResult>>,
) {
    let item_list_copy = item_list.to_owned();
    let target_path = window_weak.unwrap().get_target_directory().to_string();
    let methods: ModelHandle<SharedString> = window_weak
        .unwrap()
        .global::<SieveComboValues>()
        .get_methods();
    let sieve_method = model_to_enum(&methods, &window_weak.unwrap().get_sieve_method());
    let directory_names: ModelHandle<SharedString> = window_weak
        .unwrap()
        .global::<SieveComboValues>()
        .get_directory_names();
    let sieve_directory_names = model_to_enum(
        &directory_names,
        &window_weak.unwrap().get_sieve_directory_names(),
    );
    for _ in 0..sieve_result_model.row_count() {
        sieve_result_model.remove(0);
    }
    sieve_result_model.push(SieveResult {
        result: SharedString::from(format!(
            "Sieving using {:?} method to {} with directories {:?}",
            sieve_method, target_path, sieve_directory_names
        )),
        color: SharedString::from("black"),
    });

    thread::spawn(move || {
        let progress_callback = |progress: String| {
            let window_weak_copy = window_weak.clone();
            window_weak_copy.upgrade_in_event_loop(move |handle| {
                if progress == "Done" {
                    handle.set_sieve_running(false);
                }
                let sieve_result_model = handle.get_sieve_result_model();
                let sieve_result_model = sieve_result_model
                    .as_any()
                    .downcast_ref::<sixtyfps::VecModel<SieveResult>>()
                    .unwrap();
                let color = if progress == "Done" {
                    SharedString::from("green")
                } else if progress.starts_with("Error") {
                    SharedString::from("red")
                } else {
                    SharedString::from("black")
                };
                let sieve_result = SieveResult {
                    result: SharedString::from(progress),
                    color,
                };
                sieve_result_model.push(sieve_result);
            });
        };
        item_list_copy.sieve(
            Path::new(&target_path),
            sieve_method,
            sieve_directory_names,
            progress_callback,
        );
    });
}
