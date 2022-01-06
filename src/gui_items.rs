use sixtyfps::Image;

use crate::item_sort_list::{FileItem, ItemList};
use crate::main_window::{ListItem, SortItem};

pub fn sort_item_from_file_item(
    file_item: &FileItem,
    item_list: &ItemList,
    image: Image,
) -> SortItem {
    let mut description = format!("{}", file_item);
    if let Some(event) = item_list.get_event(file_item) {
        description = description + " \u{1F4C5} " + &event.name;
    }
    SortItem {
        text: sixtyfps::SharedString::from(description),
        image,
        take_over: file_item.get_take_over(),
        local_index: item_list.index_of_item(file_item).unwrap() as i32,
    }
}

pub fn list_item_title(file_item: &FileItem, item_list: &ItemList) -> sixtyfps::SharedString {
    let mut title = file_item.get_item_string(&item_list.path);
    if item_list.get_event(file_item).is_some() {
        title = String::from("\u{1F4C5} ") + &title;
    }
    sixtyfps::SharedString::from(title)
}

pub fn list_item_from_file_item(file_item: &FileItem, item_list: &ItemList) -> ListItem {
    ListItem {
        text: list_item_title(file_item, item_list),
        local_index: item_list.index_of_item(file_item).unwrap() as i32,
    }
}
