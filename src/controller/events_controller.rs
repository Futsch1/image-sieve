use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use sixtyfps::{Model, SharedString};

use crate::{
    item_sort_list::{parse_date, ItemList},
    main_window,
};

use super::helper;

pub struct EventsController {
    item_list: Arc<Mutex<ItemList>>,
    events_model: Rc<sixtyfps::VecModel<main_window::Event>>,
}

impl EventsController {
    pub fn new(item_list: Arc<Mutex<ItemList>>) -> Self {
        Self {
            item_list,
            events_model: Rc::new(sixtyfps::VecModel::<main_window::Event>::default()),
        }
    }

    /// Synchronize the event list with the GUI model
    pub fn synchronize(&mut self) {
        let item_list = self.item_list.lock().unwrap();
        let model_count = self.events_model.row_count();
        // Event model
        for (index, event) in item_list.events.iter().enumerate() {
            let _event = main_window::Event {
                name: SharedString::from(event.name.clone()),
                start_date: SharedString::from(event.start_date_as_string()),
                end_date: SharedString::from(event.end_date_as_string()),
            };
            if index >= model_count {
                self.events_model.push(_event);
            } else {
                self.events_model.set_row_data(index, _event);
            }
        }
    }

    /// Check the validity of an event
    pub fn check_event(&self, start_date: &str, end_date: &str, new_event: bool) -> SharedString {
        let start_date = parse_date(start_date).unwrap();
        let end_date = parse_date(end_date).unwrap();
        if start_date > end_date {
            return SharedString::from("Start date must be before end date");
        }
        let item_list = self.item_list.lock().unwrap();
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
    pub fn add_event(&mut self, name: &str, start_date: &str, end_date: &str) -> bool {
        if let Ok(event) =
            crate::item_sort_list::Event::new(String::from(name), start_date, end_date)
        {
            {
                let mut item_list = self.item_list.lock().unwrap();
                item_list.events.push(event);
                item_list.events.sort_unstable();
            }
            self.synchronize();
            true
        } else {
            false
        }
    }

    /// Update an event from the events model to the item list
    pub fn update_event(&mut self, index: i32) -> bool {
        let index = index as usize;
        let event = self.events_model.row_data(index);
        let updated = {
            let mut item_list = self.item_list.lock().unwrap();
            if item_list.events[index].update(
                event.name.to_string(),
                event.start_date.as_str(),
                event.end_date.as_str(),
            ) {
                item_list.events.sort_unstable();
                true
            } else {
                false
            }
        };
        if updated {
            self.synchronize();
        }
        updated
    }

    /// Removes an event from the item list and the events model
    pub fn remove_event(&mut self, index: i32) {
        let mut item_list = self.item_list.lock().unwrap();
        item_list.events.remove(index as usize);
        self.events_model.remove(index as usize);
    }

    /// Returns the contained sixtyfps VecModel
    pub fn get_model(&self) -> Rc<sixtyfps::VecModel<main_window::Event>> {
        self.events_model.clone()
    }

    /// Clear the events model
    pub fn clear(&mut self) {
        helper::clear_model(self.events_model.clone());
    }
}
