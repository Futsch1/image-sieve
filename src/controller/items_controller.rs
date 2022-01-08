use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use sixtyfps::Model;

use crate::{
    item_sort_list::{FileItem, ItemList},
    main_window,
    misc::image_cache,
};

use super::helper;

pub struct ItemsController {
    item_list: Arc<Mutex<ItemList>>,
    list_model: Rc<sixtyfps::VecModel<main_window::ListItem>>,
    similar_items_model: Rc<sixtyfps::VecModel<main_window::SortItem>>,
    image_cache: image_cache::ImageCache,
}

impl ItemsController {
    /// Create a new items controller instance
    pub fn new(item_list: Arc<Mutex<ItemList>>) -> Self {
        let mut image_cache = image_cache::ImageCache::new();
        image_cache.restrict_size(1600, 1000);

        Self {
            item_list,
            list_model: Rc::new(sixtyfps::VecModel::<main_window::ListItem>::default()),
            similar_items_model: Rc::new(sixtyfps::VecModel::<main_window::SortItem>::default()),
            image_cache,
        }
    }

    /// Gets the sixtyfps vec model for the item list
    pub fn get_list_model(&self) -> Rc<sixtyfps::VecModel<main_window::ListItem>> {
        self.list_model.clone()
    }

    /// Gets the sixtyfps vec model for the similar items
    pub fn get_similar_items_model(&self) -> Rc<sixtyfps::VecModel<main_window::SortItem>> {
        self.similar_items_model.clone()
    }

    /// Clear the list model
    pub fn clear_list(&mut self) {
        helper::clear_model(self.list_model.clone());
    }

    /// Clear the similar items model
    pub fn clear_similar_items(&mut self) {
        helper::clear_model(self.similar_items_model.clone());
    }

    /// Notifies that a model from the list was selected and performs all necessary actions
    /// to fill the similar items model and the current image
    pub fn selected_list_item(
        &mut self,
        list_model_index: usize,
        window: sixtyfps::Weak<main_window::ImageSieve>,
    ) {
        {
            // Clear images model
            self.clear_similar_items();

            let items_index = self.list_model.row_data(list_model_index).local_index as usize;
            let item_list = self.item_list.lock().unwrap();
            let similars = item_list.items[items_index].get_similars();

            // Clear pending commands in the image cache
            self.image_cache.purge();

            let item = &item_list.items[items_index];
            let image = self.get_item_image(
                item,
                0,
                items_index as i32,
                true,
                !similars.is_empty(),
                window.clone(),
            );
            let sort_image = sort_item_from_file_item(item, &item_list, image);
            self.similar_items_model.push(sort_image);

            let mut model_index = 1;
            for image_index in similars {
                let item = &item_list.items[*image_index];
                let image = self.get_item_image(
                    item,
                    model_index,
                    items_index as i32,
                    false,
                    !similars.is_empty(),
                    window.clone(),
                );
                let sort_image = sort_item_from_file_item(item, &item_list, image);
                self.similar_items_model.push(sort_image);
                model_index += 1;
            }
        }

        // Set properties
        window
            .unwrap()
            .set_current_image(self.similar_items_model.row_data(0));

        // And prefetch the next images
        self.prefetch_images(list_model_index);
    }

    /// Sets the take over state of an item
    pub fn set_take_over(&mut self, local_index: i32, take_over: bool) {
        {
            // Change the item_list state
            let mut item_list = self.item_list.lock().unwrap();
            item_list.items[local_index as usize].set_take_over(take_over);
        }
        // Update item list model to reflect change in icons in list
        self.update_list_model();
        // And update the take over state in the similar items model
        for count in 0..self.similar_items_model.row_count() {
            let mut item: main_window::SortItem = self.similar_items_model.row_data(count);
            if item.local_index == local_index {
                item.take_over = take_over;
                self.similar_items_model.set_row_data(count, item);
                break;
            }
        }
    }

    /// Update the texts for all entries in the list model
    /// Should be called when the underlying data (i.e. the item list) has changed
    pub fn update_list_model(&mut self) {
        let item_list = self.item_list.lock().unwrap();
        for count in 0..self.list_model.row_count() {
            let mut list_item = self.list_model.row_data(count);
            let file_item = &item_list.items[list_item.local_index as usize];
            list_item.text = list_item_title(file_item, &item_list);
            self.list_model.set_row_data(count, list_item);
        }
    }

    /// Fills the list of found items from the internal data structure to the sixtyfps VecModel
    pub fn populate_list_model(&mut self, filters: &main_window::Filters) -> usize {
        let item_list = self.item_list.lock().unwrap();
        let mut filtered_list: Vec<&FileItem> = item_list
            .items
            .iter()
            .filter(|item| filter_file_items(item, filters))
            .collect();
        filtered_list.sort_unstable_by(|a, b| compare_file_items(a, b, filters));
        if filters.direction == "Desc" {
            filtered_list.reverse();
        }
        let list_len = filtered_list.len();
        for image in filtered_list {
            let list_item = list_item_from_file_item(image, &item_list);
            self.list_model.push(list_item);
        }
        list_len
    }

    /// Gets the image for an item
    /// This function returns either a cached image or a loading image while the real image is being loaded
    /// in the background. As soon as the process finishes, the image is displayed.
    fn get_item_image(
        &self,
        item: &FileItem,
        model_index: usize,
        current_item_local_index: i32,
        is_current_image: bool,
        has_similars: bool,
        window_weak: sixtyfps::Weak<main_window::ImageSieve>,
    ) -> sixtyfps::Image {
        let image = self.image_cache.get(item);
        if let Some(image) = image {
            image
        } else {
            let f: image_cache::DoneCallback = Box::new(move |image_buffer| {
                window_weak.clone().upgrade_in_event_loop(move |handle| {
                    // Check if still the image is visible that caused the image loads
                    if handle.get_current_image().local_index == current_item_local_index {
                        let mut row_data = handle.get_similar_images_model().row_data(model_index);
                        if has_similars {
                            row_data.image = crate::misc::images::get_sixtyfps_image(&image_buffer);
                            handle
                                .get_similar_images_model()
                                .set_row_data(model_index, row_data);
                        }
                        // If the image is the current image, then we need to also update the current image SortImage
                        if is_current_image {
                            let mut current_image = handle.get_current_image();
                            current_image.image =
                                crate::misc::images::get_sixtyfps_image(&image_buffer);
                            handle.set_current_image(current_image);
                        }
                    }
                })
            });
            self.image_cache.load(
                item,
                if is_current_image {
                    image_cache::Purpose::CurrentImage
                } else {
                    image_cache::Purpose::SimilarImage
                },
                Some(f),
            );
            self.image_cache.get_waiting()
        }
    }

    /// Prefetch the next images in the model list
    fn prefetch_images(&self, list_model_index: usize) {
        // Prefetch next two images
        for i in list_model_index + 1..list_model_index + 3 {
            if i < self.list_model.row_count() {
                let item_list = self.item_list.lock().unwrap();
                let list_item = &self.list_model.row_data(i);
                let file_item = &item_list.items[list_item.local_index as usize];
                if file_item.is_image() {
                    self.image_cache
                        .load(file_item, image_cache::Purpose::Prefetch, None);
                }
            }
        }
    }
}

/// Filter file items to display in the item list
fn filter_file_items(file_item: &FileItem, filters: &main_window::Filters) -> bool {
    let mut visible = true;
    if !filters.images && file_item.is_image() {
        visible = false;
    }
    if !filters.videos && file_item.is_video() {
        visible = false;
    }
    if !filters.sorted_out && !file_item.get_take_over() {
        visible = false;
    }
    visible
}

/// Compare two file items taking the current sort settings into account
fn compare_file_items(
    a: &FileItem,
    b: &FileItem,
    filters: &main_window::Filters,
) -> std::cmp::Ordering {
    match filters.sort_by.as_str() {
        "Date" => a.cmp(b),
        "Name" => a.path.cmp(&b.path),
        "Type" => {
            if a.is_image() && b.is_image() {
                a.cmp(b)
            } else if a.is_image() && b.is_video() {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        }
        "Size" => a.get_size().cmp(&b.get_size()),
        _ => panic!("Unknown sort by type"),
    }
}

/// Create a sort item for the GUI from a file item
fn sort_item_from_file_item(
    file_item: &FileItem,
    item_list: &ItemList,
    image: sixtyfps::Image,
) -> main_window::SortItem {
    let mut description = format!("{}", file_item);
    if let Some(event) = item_list.get_event(file_item) {
        description = description + ", \u{1F4C5} " + &event.name;
    }
    main_window::SortItem {
        text: sixtyfps::SharedString::from(description),
        image,
        take_over: file_item.get_take_over(),
        local_index: item_list.index_of_item(file_item).unwrap() as i32,
    }
}

/// Get the list item title for the GUI from a file item
fn list_item_title(file_item: &FileItem, item_list: &ItemList) -> sixtyfps::SharedString {
    let mut title = file_item.get_item_string(&item_list.path);
    if item_list.get_event(file_item).is_some() {
        title = String::from("\u{1F4C5} ") + &title;
    }
    sixtyfps::SharedString::from(title)
}

/// Create a list item for the GUI from a file item
fn list_item_from_file_item(file_item: &FileItem, item_list: &ItemList) -> main_window::ListItem {
    main_window::ListItem {
        text: list_item_title(file_item, item_list),
        local_index: item_list.index_of_item(file_item).unwrap() as i32,
    }
}
