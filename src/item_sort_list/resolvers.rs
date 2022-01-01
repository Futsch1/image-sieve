extern crate chrono;
extern crate exif;
extern crate ffmpeg_next as ffmpeg;

use self::chrono::NaiveDateTime;
use self::exif::{In, Tag};

use super::item_traits::{Orientation, PropertyResolver};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub fn get_resolver(path: &Path) -> Box<dyn PropertyResolver> {
    match path.extension() {
        Some(extension) => {
            let extension = extension.to_ascii_lowercase();
            let extension_str = extension.to_str().unwrap();
            if ExifResolver::get_extensions().contains(&extension_str) {
                Box::new(ExifResolver::new(path))
            } else if FFmpegResolver::get_extensions().contains(&extension_str) {
                Box::new(FFmpegResolver::new(path))
            } else {
                Box::new(FileResolver::new(path))
            }
        }
        None => Box::new(FileResolver::new(path)),
    }
}

pub fn init_resolvers() {
    FFmpegResolver::init();
}

pub struct FileResolver {
    path: PathBuf,
}

impl FileResolver {
    pub fn new(path: &Path) -> Self {
        Self {
            path: PathBuf::from(path),
        }
    }
}

impl PropertyResolver for FileResolver {
    fn get_timestamp(&self) -> i64 {
        match std::fs::metadata(&self.path) {
            Ok(metadata) => {
                let created = metadata.created().unwrap_or_else(|_| SystemTime::now());
                let modified = metadata.modified().unwrap_or_else(|_| SystemTime::now());
                created
                    .min(modified)
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64
                    + chrono::Local::now().offset().local_minus_utc() as i64
            }
            Err(_) => 0,
        }
    }

    fn get_orientation(&self) -> Option<Orientation> {
        None
    }
}

struct ExifResolver {
    exif: Option<exif::Exif>,
    path: PathBuf,
}

impl ExifResolver {
    pub fn new(path: &Path) -> Self {
        let file = std::fs::File::open(path);
        let result = match file {
            Ok(file) => {
                let mut bufreader = std::io::BufReader::new(&file);
                let exif_reader = exif::Reader::new();
                exif_reader.read_from_container(&mut bufreader).ok()
            }
            Err(_) => None,
        };
        Self {
            exif: result,
            path: PathBuf::from(path),
        }
    }

    pub fn get_extensions() -> [&'static str; 1] {
        ["jpg"]
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

struct FFmpegResolver {
    path: PathBuf,
}

impl FFmpegResolver {
    pub fn new(path: &Path) -> Self {
        Self {
            path: PathBuf::from(path),
        }
    }

    pub fn init() {
        ffmpeg::init().ok();
    }

    pub fn get_extensions() -> [&'static str; 4] {
        ["mp4", "avi", "mts", "mov"]
    }
}

impl PropertyResolver for FFmpegResolver {
    fn get_timestamp(&self) -> i64 {
        let file_resolver = FileResolver::new(&self.path);
        if let Ok(context) = ffmpeg::format::input(&self.path) {
            for (k, v) in context.metadata().iter() {
                if k == "creation_time" {
                    if let Ok(date_time) = NaiveDateTime::parse_from_str(v, "%+") {
                        return date_time.timestamp();
                    }
                }
            }
        }
        file_resolver.get_timestamp()
    }

    fn get_orientation(&self) -> Option<Orientation> {
        if let Ok(context) = ffmpeg::format::input(&self.path) {
            if let Some(video_stream) = context.streams().best(ffmpeg::media::Type::Video) {
                for (k, v) in video_stream.metadata().iter() {
                    if k == "rotate" {
                        let orientation = match v {
                            "0" => Orientation::Landscape,
                            "90" => Orientation::Portrait90,
                            "270" => Orientation::Portrait270,
                            "180" => Orientation::Landscape180,
                            _ => Orientation::Landscape,
                        };
                        return Some(orientation);
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_timestamp_from(path: &str) -> i64 {
        get_resolver(Path::new(path)).get_timestamp()
    }

    fn get_file_timestamp(path: &str) -> i64 {
        FileResolver::new(Path::new(path)).get_timestamp()
    }

    #[test]
    fn resolvers() {
        init_resolvers();

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
            get_file_timestamp("tests/test.mp4"),
            get_timestamp_from("tests/test.mp4")
        );
        assert_eq!(1640790497, get_timestamp_from("tests/test2.mp4"));
    }
}
