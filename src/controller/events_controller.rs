use std::{
    rc::Rc,
    sync::{Arc, Mutex},
};

use slint::{Model, SharedString};

use crate::{
    item_sort_list::{self, parse_date, ItemList},
    main_window,
};

use super::helper;

pub struct EventsController {
    item_list: Arc<Mutex<ItemList>>,
    events_model: Rc<slint::VecModel<main_window::Event>>,
}

impl EventsController {
    pub fn new(item_list: Arc<Mutex<ItemList>>) -> Self {
        Self {
            item_list,
            events_model: Rc::new(slint::VecModel::<main_window::Event>::default()),
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

    /// Add an event to the item list and to the events model and sorts the lists
    pub fn add_event(&mut self, name: &str, start_date: &str, end_date: &str) -> SharedString {
        if let Err(error) = self.check_event(start_date, end_date, None) {
            error
        } else {
            let event = item_sort_list::Event::new(name, start_date, end_date);
            {
                let mut item_list = self.item_list.lock().unwrap();
                item_list.events.push(event);
                item_list.events.sort_unstable();
            }
            self.synchronize();
            SharedString::from("")
        }
    }

    /// Update an event from the events model to the item list
    pub fn update_event(
        &mut self,
        index: i32,
        name: &str,
        start_date: &str,
        end_date: &str,
    ) -> SharedString {
        let index = index as usize;
        if let Err(error) = self.check_event(start_date, end_date, Some(index)) {
            error
        } else {
            {
                let mut item_list = self.item_list.lock().unwrap();
                assert!(item_list.events[index].update(name, start_date, end_date));
                item_list.events.sort_unstable();
            };
            self.synchronize();
            SharedString::from("")
        }
    }

    /// Removes an event from the item list and the events model
    pub fn remove_event(&mut self, index: i32) {
        let mut item_list = self.item_list.lock().unwrap();
        item_list.events.remove(index as usize);
        self.events_model.remove(index as usize);
    }

    /// Returns the contained slint VecModel
    pub fn get_model(&self) -> Rc<slint::VecModel<main_window::Event>> {
        self.events_model.clone()
    }

    /// Clear the events model
    pub fn clear(&mut self) {
        helper::clear_model(self.events_model.clone());
    }

    /// Check the validity of an event
    fn check_event(
        &self,
        start_date: &str,
        end_date: &str,
        event_index: Option<usize>,
    ) -> Result<(), SharedString> {
        let start_date = parse_date(start_date);
        if let Err(start_date) = start_date {
            return Err(SharedString::from(format!("Start date: {}", start_date)));
        }
        let start_date = start_date.unwrap();

        let end_date = parse_date(end_date);
        if let Err(end_date) = end_date {
            return Err(SharedString::from(format!("End date: {}", end_date)));
        }
        let end_date = end_date.unwrap();

        if start_date > end_date {
            return Err(SharedString::from("Start date must be before end date"));
        }

        let item_list = self.item_list.lock().unwrap();
        for (index, event) in item_list.events.iter().enumerate() {
            if event_index.is_some() && index == event_index.unwrap() {
                continue;
            }
            if event.contains(&start_date) || event.contains(&end_date) {
                return Err(SharedString::from(
                    String::from("Event overlaps with ") + &event.name,
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;

    use super::*;

    #[test]
    fn test_synchronize() {
        let item_list = Arc::new(Mutex::new(ItemList::new()));
        let mut events_controller = EventsController::new(item_list.clone());
        {
            let mut item_list = item_list.lock().unwrap();
            item_list.events.push(item_sort_list::Event::new(
                "Event 1",
                "2020-01-01",
                "2020-01-02",
            ));
            item_list.events.push(item_sort_list::Event::new(
                "Event 2",
                "2020-02-01",
                "2020-02-02",
            ));
        }
        events_controller.synchronize();
        let events_model = events_controller.get_model();
        assert_eq!(events_model.row_count(), 2);
        assert_eq!(events_model.row_data(0).unwrap().name.as_str(), "Event 1");
        assert_eq!(
            events_model.row_data(0).unwrap().start_date.as_str(),
            "2020-01-01"
        );
        assert_eq!(
            events_model.row_data(0).unwrap().end_date.as_str(),
            "2020-01-02"
        );
        assert_eq!(events_model.row_data(1).unwrap().name.as_str(), "Event 2");
        assert_eq!(
            events_model.row_data(1).unwrap().start_date.as_str(),
            "2020-02-01"
        );
        assert_eq!(
            events_model.row_data(1).unwrap().end_date.as_str(),
            "2020-02-02"
        );
    }

    #[test]
    fn test_update() {
        let item_list = Arc::new(Mutex::new(ItemList::new()));
        let mut events_controller = EventsController::new(item_list.clone());
        events_controller.add_event("Event 1", "2020-01-01", "2020-01-02");

        assert_eq!(
            events_controller
                .update_event(0, "Event 11", "2020-13-03", "2020-01-04")
                .as_str(),
            "Start date: Invalid date 2020-13-03"
        );
        assert_eq!(
            events_controller
                .update_event(0, "Event 12", "2020-01-03", "01-01-2004")
                .as_str(),
            "End date: Invalid date 01-01-2004"
        );
        assert_eq!(
            events_controller
                .update_event(0, "Event 13", "2020-01-03", "2020-01-04")
                .as_str(),
            ""
        );
        let events_model = events_controller.get_model();
        assert_eq!(events_model.row_count(), 1);
        assert_eq!(events_model.row_data(0).unwrap().name.as_str(), "Event 13");
        assert_eq!(
            events_model.row_data(0).unwrap().start_date.as_str(),
            "2020-01-03"
        );
        assert_eq!(
            events_model.row_data(0).unwrap().end_date.as_str(),
            "2020-01-04"
        );
        {
            let item_list = item_list.lock().unwrap();
            assert_eq!(item_list.events[0].name.as_str(), "Event 13");
            assert_eq!(item_list.events[0].start_date.day(), 3);
            assert_eq!(item_list.events[0].end_date.day(), 4);
        }

        events_controller.add_event("Event 2", "2021-01-01", "2021-01-02");
        assert_eq!(
            events_controller
                .update_event(1, "Event 2", "2020-01-02", "2020-01-03")
                .as_str(),
            "Event overlaps with Event 13"
        );
        assert_eq!(
            events_controller
                .update_event(1, "Event 2", "2020-01-04", "2020-01-06")
                .as_str(),
            "Event overlaps with Event 13"
        );
        assert_eq!(
            events_controller
                .update_event(0, "Event 1", "2020-01-02", "2020-01-01",)
                .as_str(),
            "Start date must be before end date"
        );

        // Test changing positions
        assert_eq!(
            events_controller
                .update_event(1, "Event 2", "2019-01-01", "2019-01-01",)
                .as_str(),
            ""
        );
        let events_model = events_controller.get_model();
        assert_eq!(events_model.row_count(), 2);
        assert_eq!(events_model.row_data(0).unwrap().name.as_str(), "Event 2");
        assert_eq!(events_model.row_data(1).unwrap().name.as_str(), "Event 13");
        {
            let item_list = item_list.lock().unwrap();
            assert_eq!(item_list.events[0].name.as_str(), "Event 2");
            assert_eq!(item_list.events[1].name.as_str(), "Event 13");
        }
    }

    #[test]
    fn test_add_remove_clear() {
        let item_list = Arc::new(Mutex::new(ItemList::new()));
        let mut events_controller = EventsController::new(item_list.clone());

        assert_eq!(
            events_controller
                .add_event("Event 1", "2020-01-01", "2020-01-02")
                .as_str(),
            ""
        );
        let events_model = events_controller.get_model();
        assert_eq!(events_model.row_count(), 1);
        assert_eq!(events_model.row_data(0).unwrap().name.as_str(), "Event 1");
        assert_eq!(
            events_model.row_data(0).unwrap().start_date.as_str(),
            "2020-01-01"
        );
        assert_eq!(
            events_model.row_data(0).unwrap().end_date.as_str(),
            "2020-01-02"
        );
        {
            let item_list = item_list.lock().unwrap();
            assert_eq!(item_list.events[0].name.as_str(), "Event 1");
            assert_eq!(item_list.events[0].start_date.day(), 1);
            assert_eq!(item_list.events[0].end_date.day(), 2);
        }

        assert_eq!(
            events_controller
                .add_event("Event 2", "2019-01-03", "2019-01-04")
                .as_str(),
            ""
        );
        let events_model = events_controller.get_model();
        assert_eq!(events_model.row_count(), 2);
        assert_eq!(events_model.row_data(0).unwrap().name.as_str(), "Event 2");
        assert_eq!(
            events_model.row_data(0).unwrap().start_date.as_str(),
            "2019-01-03"
        );
        assert_eq!(
            events_model.row_data(0).unwrap().end_date.as_str(),
            "2019-01-04"
        );
        {
            let item_list = item_list.lock().unwrap();
            assert_eq!(item_list.events[0].name.as_str(), "Event 2");
        }

        assert_eq!(
            events_controller
                .add_event("Event 3", "2020-13-03", "2020-01-02")
                .as_str(),
            "Start date: Invalid date 2020-13-03"
        );
        assert_eq!(
            events_controller
                .add_event("Event 3", "2020-01-01", "2021-01-01")
                .as_str(),
            "Event overlaps with Event 1"
        );

        events_controller.remove_event(1);
        let events_model = events_controller.get_model();
        assert_eq!(events_model.row_count(), 1);
        assert_eq!(events_model.row_data(0).unwrap().name.as_str(), "Event 2");

        events_controller.clear();
        assert_eq!(events_controller.get_model().row_count(), 0);
    }
}
