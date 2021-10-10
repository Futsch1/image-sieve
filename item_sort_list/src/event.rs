extern crate chrono;

use self::chrono::NaiveDate;

pub const EVENT_DATE_FORMAT: &str = "%Y-%m-%d";

#[derive(Clone)]
pub struct Event {
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

impl Event {
    pub fn new(name: String, start_date: &str, end_date: &str) -> Result<Self, String> {
        let start_date = Event::parse_date(start_date)?;
        let end_date = Event::parse_date(end_date)?;
        Ok(Self {
            name,
            start_date: start_date,
            end_date: end_date,
        })
    }

    pub fn update(&mut self, name: String, start_date: &str, end_date: &str) -> bool {
        let start_date = Event::parse_date(start_date);
        let end_date = Event::parse_date(end_date);
        if start_date.is_ok() && end_date.is_ok() {
            self.start_date = start_date.unwrap();
            self.end_date = end_date.unwrap();
            self.name = name;
            true
        } else {
            false
        }
    }

    pub fn is_date_valid(date: &str) -> bool {
        Self::parse_date(date).is_ok()
    }

    fn parse_date(date: &str) -> Result<NaiveDate, String> {
        let possible_fmts = [EVENT_DATE_FORMAT, "%Y-%_m-%_d", "%d.%m.%Y", "%_d.%_m.%Y"];
        for fmt in possible_fmts {
            match chrono::NaiveDate::parse_from_str(&date, fmt) {
                Ok(parsed_date) => return Ok(parsed_date),
                Err(_) => (),
            };
        }
        Err(format!("Cannot parse string {}", date))
    }

    pub fn start_date_as_string(&self) -> String {
        self.start_date.format(EVENT_DATE_FORMAT).to_string()
    }

    pub fn end_date_as_string(&self) -> String {
        self.end_date.format(EVENT_DATE_FORMAT).to_string()
    }
}

mod tests {
    #[test]
    fn parse() {
        use super::Event;

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
                Event::parse_date(input)
                    .unwrap()
                    .format("%Y-%m-%d")
                    .to_string(),
                result
            );
        }

        assert!(Event::parse_date("invalid").is_err());
    }
}
