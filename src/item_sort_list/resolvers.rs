extern crate chrono;
extern crate exif;

use self::chrono::NaiveDateTime;
use self::exif::{In, Tag};

use super::item_traits::{Orientation, PropertyResolver};
use std::path::Path;
use std::time::SystemTime;

pub fn get_resolver(path: &str) -> Box<dyn PropertyResolver> {
    match Path::new(path).extension() {
        Some(extension) => {
            let extension = extension.to_ascii_lowercase();
            let extension_str = extension.to_str().unwrap();
            match extension_str {
                "jpg" => Box::new(ExifResolver::new(path)),
                _ => Box::new(FileResolver::new(path)),
            }
        }
        None => Box::new(FileResolver::new(path)),
    }
}

pub struct FileResolver {
    path: String,
}

impl FileResolver {
    pub fn new(path: &str) -> Self {
        Self {
            path: String::from(path),
        }
    }
}

impl PropertyResolver for FileResolver {
    fn get_timestamp(&self) -> i64 {
        match std::fs::metadata(&self.path) {
            Ok(metadata) => match metadata.created() {
                Ok(created) => created
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
                Err(_) => 0,
            },
            Err(_) => 0,
        }
    }

    fn get_orientation(&self) -> Option<Orientation> {
        None
    }
}

struct ExifResolver {
    exif: Option<exif::Exif>,
    path: String,
}

impl ExifResolver {
    pub fn new(path: &str) -> Self {
        let file = std::fs::File::open(path).expect("Error reading file");
        let mut bufreader = std::io::BufReader::new(&file);
        let exif_reader = exif::Reader::new();
        let result = exif_reader.read_from_container(&mut bufreader).ok();
        Self {
            exif: result,
            path: String::from(path),
        }
    }
}

impl PropertyResolver for ExifResolver {
    fn get_timestamp(&self) -> i64 {
        let file_resolver = FileResolver::new(&self.path);
        match &self.exif {
            Some(exif) => {
                let date_time_field = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY);
                match date_time_field {
                    Some(field) => {
                        let date_time_str = field.display_value().to_string();
                        let date_time =
                            NaiveDateTime::parse_from_str(&date_time_str, "%Y-%m-%d %H:%M:%S")
                                .unwrap();
                        date_time.timestamp()
                    }
                    None => file_resolver.get_timestamp(),
                }
            }
            None => file_resolver.get_timestamp(),
        }
    }

    fn get_orientation(&self) -> Option<Orientation> {
        match &self.exif {
            Some(exif) => {
                let orientation_field: Option<&exif::Field> =
                    exif.get_field(Tag::Orientation, In::PRIMARY);

                if let Some(orientation_value) = orientation_field {
                    let orientation_value = orientation_value.value.get_uint(0).unwrap();
                    let orientation = match orientation_value {
                        1 => Orientation::Landscape,
                        6 => Orientation::Portrait90,
                        8 => Orientation::Portrait270,
                        3 => Orientation::Landscape180,
                        _ => Orientation::Landscape,
                    };
                    Some(orientation)
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_timestamp_from(path: &str) -> i64 {
        get_resolver(path).get_timestamp()
    }

    fn get_file_timestamp(path: &str) -> i64 {
        FileResolver::new(path).get_timestamp()
    }

    #[test]
    fn resolvers() {
        assert_eq!(1631461311, get_timestamp_from("tests/test.jpg"));
        assert_eq!(
            get_file_timestamp("tests/test_no_date.jpg"),
            get_timestamp_from("tests/test_no_date.jpg")
        );
        assert_eq!(
            get_file_timestamp("tests/test_no_exif.jpg"),
            get_timestamp_from("tests/test_no_exif.jpg")
        );
        assert_eq!(
            get_file_timestamp("tests/test"),
            get_timestamp_from("tests/test")
        );
    }
}
