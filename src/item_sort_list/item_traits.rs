#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Orientation {
    Landscape,
    Portrait90,
    Landscape180,
    Portrait270,
}

pub trait PropertyResolver {
    fn get_timestamp(&self) -> i64;
    fn get_orientation(&self) -> Option<Orientation>;
}
