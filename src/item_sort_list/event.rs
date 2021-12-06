extern crate chrono;

use serde::{Deserialize, Serialize};

use self::chrono::NaiveDate;

pub const EVENT_DATE_FORMAT: &str = "%Y-%m-%d";

/// An event representing a name and a start and end date
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    /// Event name
    pub name: String,
    /// Event start date
    pub start_date: NaiveDate,
    /// Event end date
    pub end_date: NaiveDate,
}

impl Event {
    /// Creates a new event if the start and end date strings have a correct format
    pub fn new(name: String, start_date: &str, end_date: &str) -> Result<Self, String> {
        let start_date = parse_date(start_date)?;
        let end_date = parse_date(end_date)?;
        Ok(Self {
            name,
            start_date,
            end_date,
        })
    }

    /// Updates an event with a new name and start and end date. If start or end date have an invalid format,
    /// return false.
    pub fn update(&mut self, name: String, start_date: &str, end_date: &str) -> bool {
        let start_date = parse_date(start_date);
        let end_date = parse_date(end_date);
        if matches!(end_date, Ok(_)) && matches!(start_date, Ok(_)) {
            self.start_date = start_date.unwrap();
            self.end_date = end_date.unwrap();
            self.name = name;
            true
        } else {
            false
        }
    }

    /// Checks if a given date is valid
    pub fn is_date_valid(date: &str) -> bool {
        parse_date(date).is_ok()
    }

    /// Returns the event start date as a string
    pub fn start_date_as_string(&self) -> String {
        self.start_date.format(EVENT_DATE_FORMAT).to_string()
    }

    /// Returns the event end date as a string
    pub fn end_date_as_string(&self) -> String {
        self.end_date.format(EVENT_DATE_FORMAT).to_string()
    }

    /// Returns whether a date is within the event
    pub fn contains(&self, date: &NaiveDate) -> bool {
        self.start_date <= *date && *date <= self.end_date
    }
}

/// Parses a date string into a NaiveDate
pub fn parse_date(date: &str) -> Result<NaiveDate, String> {
    let possible_fmts = [EVENT_DATE_FORMAT, "%Y-%_m-%_d", "%d.%m.%Y", "%_d.%_m.%Y"];
    for fmt in possible_fmts {
        if let Ok(parsed_date) = chrono::NaiveDate::parse_from_str(date, fmt) {
            return Ok(parsed_date);
        }
    }
    Err(format!("Cannot parse string {}", date))
}

mod tests {
    #[cfg(test)]
    use super::*;

    #[test]
    fn test_parse() {
        let test_cases = [
            ("2021-09-14", "2021-09-14"),
            ("2021-9-14", "2021-09-14"),
            ("2021-9-4", "2021-09-04"),
            ("14.09.2021", "2021-09-14"),
            ("04.09.2021", "2021-09-04"),
            ("4.9.2021", "2021-09-04"),
        ];

        for (input, result) in test_cases {
            assert_eq!(
                parse_date(input).unwrap().format("%Y-%m-%d").to_string(),
                result
            );
        }

        assert!(parse_date("invalid").is_err());
    }

    #[test]
    fn test_as_string() {
        let event = Event::new("test".to_string(), "2021-09-14", "2021-09-15").unwrap();

        assert_eq!(event.start_date_as_string(), "2021-09-14");
        assert_eq!(event.end_date_as_string(), "2021-09-15");
    }

    #[test]
    fn test_create_and_update() {
        let event = Event::new("test".to_string(), "20-13-14", "2021-09-14");
        assert!(event.is_err());

        let event = Event::new("test".to_string(), "2021-09-14", "2021.09-14");
        assert!(event.is_err());

        let mut event = Event::new("test".to_string(), "2021-09-14", "2021-09-15").unwrap();

        assert!(event.update("test2".to_string(), "2021-09-16", "2021-09-17",));
        assert_eq!(event.name, "test2");
        assert_eq!(event.start_date_as_string(), "2021-09-16");
        assert_eq!(event.end_date_as_string(), "2021-09-17");

        assert!(!event.update("test3".to_string(), "20-09.16", "2021-09-18",));
        assert!(!event.update("test3".to_string(), "2021-09-19", "2021-09",));

        assert_eq!(event.name, "test2");
        assert_eq!(event.start_date_as_string(), "2021-09-16");
        assert_eq!(event.end_date_as_string(), "2021-09-17");
    }

    #[test]
    fn test_contains() {
        let event = Event::new("test".to_string(), "2021-09-14", "2021-09-16").unwrap();

        assert!(event.contains(&NaiveDate::from_ymd(2021, 9, 14)));
        assert!(event.contains(&NaiveDate::from_ymd(2021, 9, 15)));
        assert!(event.contains(&NaiveDate::from_ymd(2021, 9, 16)));
        assert!(!event.contains(&NaiveDate::from_ymd(2021, 9, 13)));
        assert!(!event.contains(&NaiveDate::from_ymd(2021, 9, 17)));
    }
}
