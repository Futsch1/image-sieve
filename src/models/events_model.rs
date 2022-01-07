use sixtyfps::{Model, SharedString};

use crate::{
    item_sort_list::{parse_date, ItemList},
    main_window,
};

/// Synchronize the event list with the GUI model
pub fn synchronize_event_model(
    item_list: &ItemList,
    event_model: &sixtyfps::VecModel<main_window::Event>,
) {
    let model_count = event_model.row_count();
    // Event model
    for (index, event) in item_list.events.iter().enumerate() {
        let _event = main_window::Event {
            name: SharedString::from(event.name.clone()),
            start_date: SharedString::from(event.start_date_as_string()),
            end_date: SharedString::from(event.end_date_as_string()),
        };
        if index >= model_count {
            event_model.push(_event);
        } else {
            event_model.set_row_data(index, _event);
        }
    }
}

/// Check the validity of an event
pub fn check_event(
    start_date: &str,
    end_date: &str,
    new_event: bool,
    item_list: &ItemList,
) -> SharedString {
    let start_date = parse_date(start_date).unwrap();
    let end_date = parse_date(end_date).unwrap();
    if start_date > end_date {
        return SharedString::from("Start date must be before end date");
    }
    let allowed_overlaps = if new_event { 0 } else { 1 };
    let mut overlaps = 0;
    for event in item_list.events.iter() {
        if event.contains(&start_date) || event.contains(&end_date) {
            overlaps += 1;
            if overlaps > allowed_overlaps {
                return SharedString::from(String::from("Event overlaps with ") + &event.name);
            }
        }
    }
    SharedString::from("")
}

/// Add an event to the item list and to the events model
pub fn add_event(
    name: &str,
    start_date: &str,
    end_date: &str,
    item_list: &mut ItemList,
    events_model: &sixtyfps::VecModel<main_window::Event>,
) -> bool {
    if let Ok(event) = crate::item_sort_list::Event::new(String::from(name), start_date, end_date) {
        item_list.events.push(event);
        item_list.events.sort_unstable();
        synchronize_event_model(item_list, events_model);
        true
    } else {
        false
    }
}

/// Update an event from the events model to the item list
pub fn update_event(
    index: i32,
    item_list: &mut ItemList,
    events_model: &sixtyfps::VecModel<main_window::Event>,
) -> bool {
    let index = index as usize;
    let event = events_model.row_data(index);
    if item_list.events[index].update(
        event.name.to_string(),
        event.start_date.as_str(),
        event.end_date.as_str(),
    ) {
        item_list.events.sort_unstable();
        synchronize_event_model(item_list, events_model);
        true
    } else {
        false
    }
}

/// Removes an event from the item list and the events model
pub fn remove_event(
    index: i32,
    item_list: &mut ItemList,
    events_model: &sixtyfps::VecModel<main_window::Event>,
) {
    item_list.events.remove(index as usize);
    events_model.remove(index as usize);
}
