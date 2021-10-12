use std::cmp::Ordering;
use std::fmt::Debug;
use std::path::Path;

use super::item_traits::Orientation;
use super::item_traits::PropertyResolver;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct FileItem {
    path: String,
    timestamp: i64,
    take_over: bool,
    similar: Vec<usize>,
    extension: String,
    orientation: Option<Orientation>,
}

impl FileItem {
    pub fn new(
        path: String,
        property_resolver: Box<dyn PropertyResolver>,
        take_over: bool,
    ) -> Self {
        let timestamp = property_resolver.get_timestamp();
        let extension = extension(&path);
        let orientation = property_resolver.get_orientation();
        Self {
            path,
            timestamp,
            take_over,
            similar: vec![],
            extension,
            orientation,
        }
    }

    #[cfg(test)]
    pub fn dummy(timestamp: i64) -> Self {
        Self {
            path: String::from(""),
            timestamp,
            orientation: None,
            take_over: false,
            similar: vec![],
            extension: String::from(""),
        }
    }

    pub fn set_take_over(&mut self, take_over: bool) {
        self.take_over = take_over;
    }

    pub fn get_take_over(&self) -> bool {
        self.take_over
    }

    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn get_date_str(&self) -> String {
        chrono::NaiveDateTime::from_timestamp(self.timestamp, 0)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
    }

    pub fn get_size(&self) -> u64 {
        self.get_path().metadata().unwrap().len()
    }

    pub fn get_path(&self) -> &Path {
        Path::new(&self.path)
    }

    pub fn get_path_as_str(&self) -> &String {
        &self.path
    }

    pub fn set_similars(&mut self, similar: Vec<usize>) {
        self.similar = similar;
    }

    pub fn get_similars(&self) -> &Vec<usize> {
        &self.similar
    }

    pub fn get_orientation(&self) -> Option<&Orientation> {
        self.orientation.as_ref()
    }

    pub fn get_item_string(&self, base_path: &str) -> String {
        let path = Path::new(&self.path);
        let path = path.strip_prefix(base_path).unwrap_or(path);
        let similars_str = if !self.get_similars().is_empty() {
            "\u{1F500} "
        } else {
            ""
        };
        let extension_str = self.extension_to_string();
        let take_over_str = if self.take_over { "" } else { "\u{1F5D1} " };
        let strings = [
            similars_str,
            extension_str,
            take_over_str,
            path.to_str().unwrap(),
        ];
        strings.join(" ")
    }

    pub fn is_image(&self) -> bool {
        matches!(self.extension.as_str(), "jpg" | "png" | "tif")
    }

    pub fn is_video(&self) -> bool {
        matches!(self.extension.as_str(), "mp4" | "avi" | "mts")
    }

    pub fn get_extensions() -> [&'static str; 6] {
        ["jpg", "png", "tif", "mp4", "avi", "mts"]
    }

    fn extension_to_string(&self) -> &str {
        if self.is_image() {
            "\u{1F4F7} "
        } else if self.is_video() {
            "\u{1F4F9} "
        } else {
            ""
        }
    }
}

impl PartialOrd for FileItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FileItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

fn extension(path: &str) -> String {
    let path = Path::new(path);
    let extension = path.extension().unwrap_or_default().to_ascii_lowercase();
    String::from(extension.to_str().unwrap())
}
