use chrono::Datelike;
use strum_macros::Display;

#[derive(Display, PartialEq, Eq)]
pub enum Format {
    #[strum(serialize = "%Y-%m-%d")]
    Date,
    #[strum(serialize = "%Y-%m-%d %H:%M:%S")]
    DateTime,
    #[strum(serialize = "%Y")]
    Year,
    #[strum(serialize = "%Y-%m")]
    YearAndMonth,
    #[strum(serialize = "%Y")]
    YearAndQuarter,
    #[strum(serialize = "%m")]
    Month,
}

pub fn timestamp_to_string(timestamp: i64, fmt: Format) -> String {
    let d = chrono::DateTime::from_timestamp(timestamp, 0);
    if let Some(d) = d {
        if fmt == Format::YearAndQuarter {
            d.format("%Y-Q").to_string() + &format!("{}", (d.date_naive().month() - 1) / 3 + 1)
        } else {
            d.format(&fmt.to_string()).to_string()
        }
    } else {
        String::from("???")
    }
}
