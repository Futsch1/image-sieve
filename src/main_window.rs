//! Module containing the main window of image_sieve

extern crate nfd;
extern crate slint;

use slint::{Model, ModelRc, SharedString};
use std::cell::RefCell;
use std::fmt::Debug;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::controller::events_controller::EventsController;
use crate::controller::items_controller::ItemsController;
use crate::item_sort_list::ItemList;
use crate::misc::images::get_empty_image;
use crate::persistence::json::{get_project_filename, get_settings_filename, JsonPersistence};
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
    slint::include_modules!();
}
pub use generated_code::*;

/// Main window container of the image sorter, contains the slint window, models and internal data structures
pub struct MainWindow {
    window: ImageSieve,
    item_list: Arc<Mutex<ItemList>>,
    items_controller: Rc<RefCell<ItemsController>>,
    events_controller: Rc<RefCell<EventsController>>,
    sieve_result_model: Rc<slint::VecModel<SieveResult>>,
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

        let events_controller = Rc::new(RefCell::new(EventsController::new(item_list.clone())));
        let items_controller = Rc::new(RefCell::new(ItemsController::new(item_list.clone())));
        let sieve_result_model = Rc::new(slint::VecModel::<SieveResult>::default());

        // Construct main window
        let image_sieve = ImageSieve::new();

        let synchronizer = Synchronizer::new(item_list.clone(), &image_sieve);
        if !settings.settings_v05.source_directory.is_empty() {
            // Start synchronization in a background thread
            synchronizer.scan_path(Path::new(&settings.settings_v05.source_directory));
        }

        let main_window = Self {
            window: image_sieve,
            item_list,
            items_controller,
            events_controller,
            sieve_result_model,
            synchronizer: Rc::new(synchronizer),
        };

        // Set initial values
        let version = env!("CARGO_PKG_VERSION");
        main_window
            .window
            .set_window_title(SharedString::from("ImageSieve v") + version);
        settings.to_window(&main_window.window);
        if settings.settings_v05.source_directory.is_empty() {
            main_window.window.set_loading(false);
            main_window.window.set_calculating_similarities(false);
        }
        /*main_window
            .window
            .set_system_dark(dark_light::detect() == dark_light::Mode::Dark);
        */

        // Set model references
        main_window.window.set_list_model(
            main_window
                .items_controller
                .borrow()
                .get_list_model()
                .into(),
        );
        main_window.window.set_similar_images_model(
            main_window
                .items_controller
                .borrow()
                .get_similar_items_model()
                .into(),
        );
        main_window
            .window
            .set_events_model(main_window.events_controller.borrow().get_model().into());
        main_window
            .window
            .set_sieve_result_model(main_window.sieve_result_model.clone().into());

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

    /// Setup slint GUI callbacks
    fn setup_callbacks(&self) {
        self.window.on_item_selected({
            // New item selected on the list of images or next/previous clicked
            let items_controller = self.items_controller.clone();
            let window_weak = self.window.as_weak();

            move |i: i32| {
                items_controller
                    .borrow_mut()
                    .selected_list_item(i as usize, window_weak.clone());
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
            let items_controller = self.items_controller.clone();

            move |i: i32, take_over: bool| -> SharedString {
                // Change the state of the SortImage in the items_model
                items_controller.borrow_mut().set_take_over(i, take_over)
            }
        });

        self.window.on_browse_source({
            // Browse source was clicked, select new path
            let events_controller = self.events_controller.clone();
            let items_controller = self.items_controller.clone();
            let item_list = self.item_list.clone();
            let window_weak = self.window.as_weak();
            let synchronizer = self.synchronizer.clone();

            move || {
                if let Ok(nfd::Response::Okay(folder)) =
                    nfd::open_pick_folder(get_folder(&window_weak.unwrap().get_source_directory()))
                {
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

                    items_controller.borrow_mut().clear_list();
                    events_controller.borrow_mut().clear();

                    // Synchronize in a background thread
                    window_weak.unwrap().set_loading(true);
                    synchronizer.scan_path(Path::new(&folder));

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
                if let Ok(nfd::Response::Okay(folder)) =
                    nfd::open_pick_folder(get_folder(&window_weak.unwrap().get_target_directory()))
                {
                    window_weak
                        .unwrap()
                        .set_target_directory(SharedString::from(folder));
                }
            }
        });

        self.window.on_synchronization_finished({
            // First step of synchronization (browsing for files) finished
            let window_weak = self.window.as_weak();
            let events_controller = self.events_controller.clone();
            let items_controller = self.items_controller.clone();
            let synchronizer = self.synchronizer.clone();

            move || {
                let window = window_weak.unwrap();
                let filters = window.get_filters();
                // First fill the list of items
                let num_items = items_controller.borrow_mut().populate_list_model(&filters);

                // Now fill the events model
                events_controller.borrow_mut().synchronize();

                // Update the selection variables
                if num_items > 0 {
                    window.set_current_list_item(0);
                    window.invoke_item_selected(0);
                } else {
                    let empty_image = SortItem {
                        image: get_empty_image(),
                        take_over: true,
                        text: SharedString::from("No images found"),
                        local_index: 0,
                    };
                    window.set_current_image(empty_image);
                    items_controller.borrow_mut().clear_similar_items();
                }

                // Show the GUI by resetting the loading flag
                window.set_loading(false);

                // And tell the synchronizer to calculate similarities now
                synchronizer.calculate_similarities(Settings::from_window(&window));
            }
        });

        self.window.on_similarities_calculated({
            // Second step of synchronization (calculating similarities) finished
            let items_controller = self.items_controller.clone();
            let window_weak = self.window.as_weak();

            move |finished| {
                let window = window_weak.unwrap();
                if items_controller.borrow_mut().update_list_model() {
                    items_controller.borrow_mut().selected_list_item(
                        window.get_current_list_item() as usize,
                        window_weak.clone(),
                    );
                }
                if finished {
                    window.set_calculating_similarities(false);
                }
            }
        });

        self.window.on_add_event({
            // New event was added, return true if the dates are ok
            let events_controller = self.events_controller.clone();
            let items_controller = self.items_controller.clone();

            move |name: SharedString,
                  start_date: SharedString,
                  end_date: SharedString|
                  -> SharedString {
                let result =
                    events_controller
                        .borrow_mut()
                        .add_event(&name, &start_date, &end_date);
                if result.is_empty() {
                    items_controller.borrow_mut().update_list_model();
                }
                result
            }
        });

        self.window.on_update_event({
            let events_controller = self.events_controller.clone();
            let items_controller = self.items_controller.clone();
            move |index: i32,
                  name: SharedString,
                  start_date: SharedString,
                  end_date: SharedString|
                  -> SharedString {
                let result = events_controller.borrow_mut().update_event(
                    index,
                    &name,
                    &start_date,
                    &end_date,
                );
                if result.is_empty() {
                    items_controller.borrow_mut().update_list_model();
                }
                result
            }
        });

        self.window.on_remove_event({
            // Event was removed
            let events_controller = self.events_controller.clone();
            let items_controller = self.items_controller.clone();

            move |index| {
                events_controller.borrow_mut().remove_event(index);
                items_controller.borrow_mut().update_list_model();
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
                let window = window_weak.unwrap();
                // Synchronize in a background thread
                window.set_calculating_similarities(true);
                synchronizer.calculate_similarities(Settings::from_window(&window));
            }
        });

        self.window.on_cancel_loading({
            let synchronizer = self.synchronizer.clone();
            move || {
                synchronizer.stop();
            }
        });

        self.window.on_filter({
            let items_controller = self.items_controller.clone();
            let window_weak = self.window.as_weak();

            move |filters| {
                let rows = items_controller.borrow_mut().populate_list_model(&filters) as i32;
                if rows <= window_weak.unwrap().get_current_list_item() {
                    window_weak.unwrap().set_current_list_item(rows - 1);
                }
            }
        });

        self.window.on_fill_event_cb({
            let items_controller = self.items_controller.clone();
            let window_weak = self.window.as_weak();

            move |local_index| {
                let date_string = items_controller.borrow().get_date_string(local_index);
                window_weak.unwrap().invoke_fill_event(date_string);
            }
        });
    }
}

/// Sieves the item list in a background thread
pub fn sieve(
    item_list: &ItemList,
    window_weak: slint::Weak<ImageSieve>,
    sieve_result_model: Rc<slint::VecModel<SieveResult>>,
) {
    let item_list_copy = item_list.to_owned();
    let target_path = window_weak.unwrap().get_target_directory().to_string();
    let methods: ModelRc<SharedString> = window_weak
        .unwrap()
        .global::<SieveComboValues>()
        .get_methods();
    let sieve_method = model_to_enum(&methods, &window_weak.unwrap().get_sieve_method());
    let directory_names: ModelRc<SharedString> = window_weak
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
            window_weak_copy
                .upgrade_in_event_loop(move |handle| {
                    if progress == "Done" {
                        handle.set_sieve_running(false);
                    }
                    let sieve_result_model = handle.get_sieve_result_model();
                    let sieve_result_model = sieve_result_model
                        .as_any()
                        .downcast_ref::<slint::VecModel<SieveResult>>()
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
                })
                .unwrap();
        };
        item_list_copy.sieve(
            Path::new(&target_path),
            sieve_method,
            sieve_directory_names,
            progress_callback,
        );
    });
}

/// Convert a folder setting to an option if the folder exists
fn get_folder(folder: &SharedString) -> Option<&str> {
    let folder = folder.as_str();
    if Path::new(folder).exists() {
        Some(folder)
    } else {
        None
    }
}
