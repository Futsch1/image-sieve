use serde::{Deserialize, Serialize};

/// Image orientation
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Orientation {
    Landscape,
    Portrait90,
    Landscape180,
    Portrait270,
}

/// Trait to get a timestamp and an optional orientation from a file
pub trait PropertyResolver {
    fn get_timestamp(&self) -> i64;
    fn get_orientation(&self) -> Option<Orientation>;
}
