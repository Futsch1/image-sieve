use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use slint::Model;

use crate::{
    item_sort_list::{FileItem, ItemList},
    main_window,
    misc::image_cache,
};

use super::helper;

pub struct ItemsController {
    item_list: Arc<Mutex<ItemList>>,
    list_model: Rc<slint::VecModel<main_window::ListItem>>,
    similar_items_model: Rc<slint::VecModel<main_window::SortItem>>,
    image_cache: image_cache::ImageCache,
}

impl ItemsController {
    /// Create a new items controller instance
    pub fn new(item_list: Arc<Mutex<ItemList>>) -> Self {
        let mut image_cache = image_cache::ImageCache::new();
        image_cache.restrict_size(1600, 1000);

        Self {
            item_list,
            list_model: Rc::new(slint::VecModel::<main_window::ListItem>::default()),
            similar_items_model: Rc::new(slint::VecModel::<main_window::SortItem>::default()),
            image_cache,
        }
    }

    /// Gets the slint vec model for the item list
    pub fn get_list_model(&self) -> Rc<slint::VecModel<main_window::ListItem>> {
        self.list_model.clone()
    }

    /// Gets the slint vec model for the similar items
    pub fn get_similar_items_model(&self) -> Rc<slint::VecModel<main_window::SortItem>> {
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
        window: slint::Weak<main_window::ImageSieve>,
    ) {
        if list_model_index >= self.list_model.row_count() {
            return;
        }
        {
            // Clear images model
            self.clear_similar_items();

            let items_index = self
                .list_model
                .row_data(list_model_index)
                .unwrap()
                .local_index as usize;
            let item_list = self.item_list.lock().unwrap();
            let similars = item_list.items[items_index].get_similars();

            // Clear pending commands in the image cache
            self.image_cache.purge();

            // Add the current image
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

            // Now add all similar images
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

        // Set the data of the current image
        window
            .unwrap()
            .set_current_image(self.similar_items_model.row_data(0).unwrap());

        // And prefetch the next images
        self.prefetch_images(list_model_index);
    }

    /// Sets the take over state of an item
    pub fn set_take_over(&mut self, local_index: i32, take_over: bool) -> slint::SharedString {
        let description = {
            // Change the item_list state
            let mut item_list = self.item_list.lock().unwrap();
            item_list.items[local_index as usize].set_take_over(take_over);
            sort_item_description(&item_list.items[local_index as usize], &item_list)
        };
        // Update item list model to reflect change in icons in list
        self.update_list_model();
        // And update the take over state in the similar items model
        for count in 0..self.similar_items_model.row_count() {
            let mut item: main_window::SortItem = self.similar_items_model.row_data(count).unwrap();
            if item.local_index == local_index {
                item.take_over = take_over;
                item.text = description.clone();
                self.similar_items_model.set_row_data(count, item);
                break;
            }
        }
        description
    }

    /// Update the texts for all entries in the list model and returns true if the list contains more than one item
    /// Should be called when the underlying data (i.e. the item list) has changed
    pub fn update_list_model(&mut self) -> bool {
        let item_list = self.item_list.lock().unwrap();
        for count in 0..self.list_model.row_count() {
            let mut list_item = self.list_model.row_data(count).unwrap();
            let file_item = &item_list.items[list_item.local_index as usize];
            list_item.text = list_item_title(file_item, &item_list);
            self.list_model.set_row_data(count, list_item);
        }
        !item_list.items.is_empty()
    }

    /// Fills the list of found items from the internal data structure to the slint VecModel
    pub fn populate_list_model(&mut self, filters: &main_window::Filters) -> usize {
        self.clear_list();

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

    /// Gets the date string for an image
    pub fn get_date_string(&self, local_index: i32) -> slint::SharedString {
        let item_list = self.item_list.lock().unwrap();
        let item = &item_list.items[local_index as usize];
        let date = chrono::NaiveDateTime::from_timestamp(item.get_timestamp(), 0)
            .format("%Y-%m-%d")
            .to_string();
        slint::SharedString::from(date)
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
        window_weak: slint::Weak<main_window::ImageSieve>,
    ) -> slint::Image {
        let image = self.image_cache.get(item);
        if let Some(image) = image {
            image
        } else {
            let f: image_cache::DoneCallback = Box::new(move |image_buffer| {
                window_weak
                    .clone()
                    .upgrade_in_event_loop(move |handle| {
                        // Check if still the image is visible that caused the image loads
                        if handle.get_current_image().local_index == current_item_local_index {
                            let mut row_data = handle
                                .get_similar_images_model()
                                .row_data(model_index)
                                .unwrap();
                            if has_similars {
                                row_data.image =
                                    crate::misc::images::get_slint_image(&image_buffer);
                                handle
                                    .get_similar_images_model()
                                    .set_row_data(model_index, row_data);
                            }
                            // If the image is the current image, then we need to also update the current image SortImage
                            if is_current_image {
                                let mut current_image = handle.get_current_image();
                                current_image.image =
                                    crate::misc::images::get_slint_image(&image_buffer);
                                handle.set_current_image(current_image);
                            }
                        }
                    })
                    .ok();
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
                let list_item = &self.list_model.row_data(i).unwrap();
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
    if !filters.images && (file_item.is_image() || file_item.is_raw_image()) {
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
    image: slint::Image,
) -> main_window::SortItem {
    main_window::SortItem {
        text: sort_item_description(file_item, item_list),
        image,
        take_over: file_item.get_take_over(),
        local_index: item_list.index_of_item(file_item).unwrap() as i32,
    }
}

/// Gets the description of a sort item from a file item
fn sort_item_description(file_item: &FileItem, item_list: &ItemList) -> slint::SharedString {
    let mut description = format!("{}", file_item);
    if let Some(event) = item_list.get_event(file_item) {
        description = description + ", 📅 " + &event.name;
    }
    slint::SharedString::from(description)
}

/// Get the list item title for the GUI from a file item
fn list_item_title(file_item: &FileItem, item_list: &ItemList) -> slint::SharedString {
    let mut title = file_item.get_item_string(&item_list.path);
    if item_list.get_event(file_item).is_some() {
        title = String::from("📅 ") + &title;
    }
    slint::SharedString::from(title)
}

/// Create a list item for the GUI from a file item
fn list_item_from_file_item(file_item: &FileItem, item_list: &ItemList) -> main_window::ListItem {
    main_window::ListItem {
        text: list_item_title(file_item, item_list),
        local_index: item_list.index_of_item(file_item).unwrap() as i32,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::MutexGuard;

    use crate::main_window::ImageSieve;
    use slint::{ComponentHandle, SharedString};

    use super::*;

    fn build_filters() -> main_window::Filters {
        main_window::Filters {
            images: true,
            videos: true,
            sorted_out: true,
            sort_by: SharedString::from("Date"),
            direction: SharedString::from("Asc"),
        }
    }

    fn fill_item_list(item_list: &mut MutexGuard<ItemList>) {
        item_list
            .items
            .push(FileItem::dummy("test2.mov", 86400, true));
        let mut file_item = FileItem::dummy("test1.jpg", 1, false);
        file_item.add_similar_range(&(0..1));
        item_list.items.push(file_item);
    }

    #[test]
    fn test_populate() {
        let item_list = Arc::new(Mutex::new(ItemList::new()));
        let mut items_controller = ItemsController::new(item_list.clone());
        let mut filters = build_filters();
        {
            let mut item_list = item_list.lock().unwrap();
            fill_item_list(&mut item_list);
        }
        items_controller.populate_list_model(&filters);
        let list_model = items_controller.get_list_model();
        assert_eq!(list_model.row_count(), 2);
        assert_eq!(list_model.row_data(0).unwrap().local_index, 1);
        assert_eq!(list_model.row_data(0).unwrap().text, "🔀 📷 🗑 test1.jpg");
        assert_eq!(list_model.row_data(1).unwrap().local_index, 0);
        assert_eq!(list_model.row_data(1).unwrap().text, "📹 test2.mov");

        filters.direction = SharedString::from("Desc");
        items_controller.populate_list_model(&filters);
        assert_eq!(list_model.row_count(), 2);
        assert_eq!(list_model.row_data(1).unwrap().local_index, 1);
        assert_eq!(list_model.row_data(1).unwrap().text, "🔀 📷 🗑 test1.jpg");
        assert_eq!(list_model.row_data(0).unwrap().local_index, 0);
        assert_eq!(list_model.row_data(0).unwrap().text, "📹 test2.mov");

        filters.images = false;
        items_controller.populate_list_model(&filters);
        assert_eq!(list_model.row_count(), 1);
        assert_eq!(list_model.row_data(0).unwrap().local_index, 0);
        assert_eq!(list_model.row_data(0).unwrap().text, "📹 test2.mov");

        filters.images = true;
        filters.videos = false;
        items_controller.populate_list_model(&filters);
        assert_eq!(list_model.row_count(), 1);
        assert_eq!(list_model.row_data(0).unwrap().local_index, 1);
        assert_eq!(list_model.row_data(0).unwrap().text, "🔀 📷 🗑 test1.jpg");

        filters.videos = true;
        filters.sorted_out = false;
        items_controller.populate_list_model(&filters);
        assert_eq!(list_model.row_count(), 1);
        assert_eq!(list_model.row_data(0).unwrap().local_index, 0);
        assert_eq!(list_model.row_data(0).unwrap().text, "📹 test2.mov");

        items_controller.clear_list();
        assert_eq!(items_controller.get_list_model().row_count(), 0);
    }

    #[test]
    fn test_take_over() {
        let item_list = Arc::new(Mutex::new(ItemList::new()));
        let mut items_controller = ItemsController::new(item_list.clone());
        let window = ImageSieve::new();
        let window_weak = window.as_weak();
        let filters = build_filters();
        {
            let mut item_list = item_list.lock().unwrap();
            fill_item_list(&mut item_list);
        }
        items_controller.populate_list_model(&filters);
        items_controller.selected_list_item(1, window_weak);

        items_controller.set_take_over(0, false);
        {
            let item_list = item_list.lock().unwrap();
            assert!(!item_list.items[0].get_take_over());
        }
        let list_model = items_controller.get_list_model();
        let similar_items_model = items_controller.get_similar_items_model();
        assert_eq!(list_model.row_data(1).unwrap().text, "📹 🗑 test2.mov");
        assert!(!similar_items_model.row_data(0).unwrap().take_over);

        items_controller.set_take_over(0, true);
        {
            let item_list = item_list.lock().unwrap();
            assert!(item_list.items[0].get_take_over());
        }
        assert_eq!(list_model.row_data(1).unwrap().text, "📹 test2.mov");
        assert!(window.get_current_image().take_over);
        assert!(similar_items_model.row_data(0).unwrap().take_over);
    }

    #[test]
    fn test_select_item() {
        let item_list = Arc::new(Mutex::new(ItemList::new()));
        let mut items_controller = ItemsController::new(item_list.clone());
        let window = ImageSieve::new();
        let window_weak = window.as_weak();
        let filters = build_filters();
        {
            let mut item_list = item_list.lock().unwrap();
            fill_item_list(&mut item_list);
        }
        items_controller.populate_list_model(&filters);

        let similar_items_model = items_controller.get_similar_items_model();

        items_controller.selected_list_item(0, window_weak.clone());
        assert_eq!(similar_items_model.row_count(), 2);
        assert_eq!(
            similar_items_model.row_data(0).unwrap().text,
            "🔀 📷 🗑 test1.jpg - 1970-01-01 00:00:01, 0 KB"
        );
        assert_eq!(
            similar_items_model.row_data(1).unwrap().text,
            "📹 test2.mov - 1970-01-02 00:00:00, 0 KB"
        );
        assert_eq!(
            similar_items_model.row_data(0).unwrap().image.size().width as i32,
            16
        );
        assert_eq!(window.get_current_image().image.size().height as i32, 16);
        assert_eq!(window.get_current_image().local_index, 1);

        items_controller.selected_list_item(1, window_weak);
        assert_eq!(similar_items_model.row_count(), 1);
        assert_eq!(window.get_current_image().local_index, 0);
    }

    #[test]
    fn test_update_list() {
        let item_list = Arc::new(Mutex::new(ItemList::new()));
        let mut items_controller = ItemsController::new(item_list.clone());
        assert!(!items_controller.update_list_model());
        let filters = build_filters();
        {
            let mut item_list = item_list.lock().unwrap();
            fill_item_list(&mut item_list);
        }
        items_controller.populate_list_model(&filters);

        {
            let mut item_list = item_list.lock().unwrap();
            item_list.events.push(crate::item_sort_list::Event::new(
                "Test",
                "1970-01-01",
                "1970-01-02",
            ));
        }
        assert!(items_controller.update_list_model());
        let list_model = items_controller.get_list_model();
        assert_eq!(list_model.row_count(), 2);
        assert_eq!(list_model.row_data(0).unwrap().text, "📅 🔀 📷 🗑 test1.jpg");
        assert_eq!(list_model.row_data(1).unwrap().text, "📅 📹 test2.mov");
    }

    #[test]
    fn test_get_date_string() {
        let item_list = Arc::new(Mutex::new(ItemList::new()));
        let items_controller = ItemsController::new(item_list.clone());
        {
            let mut item_list = item_list.lock().unwrap();
            fill_item_list(&mut item_list);
        }
        assert_eq!(items_controller.get_date_string(0), "1970-01-02");
        assert_eq!(items_controller.get_date_string(1), "1970-01-01");
    }
}
