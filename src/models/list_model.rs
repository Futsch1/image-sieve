use sixtyfps::Model;

use crate::item_sort_list::{FileItem, ItemList};
use crate::main_window::{Filters, ListItem};

use super::gui_items::{list_item_from_file_item, list_item_title};

/// Fills the list of found items from the internal data structure to the sixtyfps VecModel
pub fn populate_list_model(
    item_list: &ItemList,
    list_model: &sixtyfps::VecModel<ListItem>,
    filters: &Filters,
) {
    let mut filtered_list: Vec<&FileItem> = item_list
        .items
        .iter()
        .filter(|item| filter_file_items(item, filters))
        .collect();
    filtered_list.sort_unstable_by(|a, b| compare_file_items(a, b, filters));
    if filters.direction == "Desc" {
        filtered_list.reverse();
    }
    for image in filtered_list {
        let list_item = list_item_from_file_item(image, item_list);
        list_model.push(list_item);
    }
}

/// Update the texts for all entries in the list model
/// Should be called when the underlying data (i.e. the item list) has changed
pub fn update_list_model(item_list: &ItemList, list_model: &sixtyfps::VecModel<ListItem>) {
    for count in 0..list_model.row_count() {
        let mut list_item = list_model.row_data(count);
        let file_item = &item_list.items[list_item.local_index as usize];
        list_item.text = list_item_title(file_item, item_list);
        list_model.set_row_data(count, list_item);
    }
}

/// Filter file items to display in the item list
fn filter_file_items(file_item: &FileItem, filters: &Filters) -> bool {
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
fn compare_file_items(a: &FileItem, b: &FileItem, filters: &Filters) -> std::cmp::Ordering {
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
