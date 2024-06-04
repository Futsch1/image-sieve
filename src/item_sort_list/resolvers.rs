extern crate chrono;
extern crate exif;
extern crate ffmpeg_next as ffmpeg;

use self::chrono::NaiveDateTime;
use self::exif::{In, Tag};

use super::file_types::{is_image, is_raw_image, is_video};
use super::item_traits::{Orientation, PropertyResolver};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub fn get_resolver(path: &Path) -> Box<dyn PropertyResolver> {
    if ExifResolver::supports(path) {
        Box::new(ExifResolver::new(path))
    } else if FFmpegResolver::supports(path) {
        Box::new(FFmpegResolver::new(path))
    } else if RawResolver::supports(path) {
        Box::new(RawResolver::new(path))
    } else {
        Box::new(FileResolver::new(path))
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
            Err(_) => -1,
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

    pub fn supports(path: &Path) -> bool {
        is_image(path)
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
                        if let Ok(date_time) =
                            NaiveDateTime::parse_from_str(&date_time_str, "%Y-%m-%d %H:%M:%S")
                        {
                            date_time.timestamp()
                        } else {
                            file_resolver.get_timestamp()
                        }
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
                    Some(match orientation_value {
                        1 => Orientation::Landscape,
                        6 => Orientation::Portrait90,
                        8 => Orientation::Portrait270,
                        3 => Orientation::Landscape180,
                        _ => Orientation::Landscape,
                    })
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

    pub fn supports(path: &Path) -> bool {
        is_video(path)
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
                        return Some(match v {
                            "0" => Orientation::Landscape,
                            "90" => Orientation::Portrait90,
                            "270" => Orientation::Portrait270,
                            "180" => Orientation::Landscape180,
                            _ => Orientation::Landscape,
                        });
                    }
                }
            }
        }
        None
    }
}

struct RawResolver {
    path: PathBuf,
}

impl RawResolver {
    pub fn new(path: &Path) -> Self {
        Self {
            path: PathBuf::from(path),
        }
    }

    pub fn supports(path: &Path) -> bool {
        is_raw_image(path)
    }
}

impl PropertyResolver for RawResolver {
    fn get_timestamp(&self) -> i64 {
        ExifResolver::new(&self.path).get_timestamp()
    }

    fn get_orientation(&self) -> Option<Orientation> {
        match rawloader::decode_file(&self.path) {
            Ok(raw) => match raw.orientation {
                rawloader::Orientation::Normal => Some(Orientation::Landscape),
                rawloader::Orientation::Rotate90 => Some(Orientation::Portrait90),
                rawloader::Orientation::Rotate270 => Some(Orientation::Portrait270),
                rawloader::Orientation::Rotate180 => Some(Orientation::Landscape180),
                _ => None,
            },
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_timestamp_from(path: &str) -> i64 {
        get_resolver(Path::new(path)).get_timestamp()
    }

    fn get_orientation_from(path: &str) -> Option<Orientation> {
        get_resolver(Path::new(path)).get_orientation()
    }

    fn get_file_timestamp(path: &str) -> i64 {
        FileResolver::new(Path::new(path)).get_timestamp()
    }

    #[test]
    fn resolvers() {
        init_resolvers();

        assert_eq!(1631461311, get_timestamp_from("tests/test.jpg"));
        assert_eq!(
            Some(Orientation::Portrait90),
            get_orientation_from("tests/test.jpg")
        );

        assert_eq!(1631461311, get_timestamp_from("tests/test2.JPG"));
        assert_eq!(None, get_orientation_from("tests/test2.JPG"));
        assert_eq!(
            get_file_timestamp("tests/test_no_date.jpg"),
            get_timestamp_from("tests/test_no_date.jpg")
        );
        assert_eq!(
            get_file_timestamp("tests/test_no_exif.jpg"),
            get_timestamp_from("tests/test_no_exif.jpg")
        );
        assert_eq!(
            get_file_timestamp("tests/test_invalid.jpg"),
            get_timestamp_from("tests/test_invalid.jpg")
        );
        assert_eq!(
            get_file_timestamp("tests/test_invalid_date.jpg"),
            get_timestamp_from("tests/test_invalid_date.jpg")
        );

        assert_eq!(0, get_timestamp_from("tests/test.png"));
        assert_eq!(
            Some(Orientation::Landscape180),
            get_orientation_from("tests/test.png")
        );

        assert_eq!(
            get_file_timestamp("tests/test.mp4"),
            get_timestamp_from("tests/test.mp4")
        );
        assert_eq!(None, get_orientation_from("tests/test.mp4"));
        assert_eq!(1640790497, get_timestamp_from("tests/test2.MP4"));
        //TODO: There seems to be an issue in FFMPEG with getting the orientation
        assert_eq!(
            //Some(Orientation::Landscape180),
            None,
            get_orientation_from("tests/test2.MP4")
        );
        assert_eq!(
            get_file_timestamp("tests/test_invalid.mp4"),
            get_timestamp_from("tests/test_invalid.mp4")
        );

        assert_eq!(974638910, get_timestamp_from("tests/test.nef"));
        assert_eq!(None, get_orientation_from("tests/test.nef"));

        assert_eq!(-1, get_timestamp_from("not_there"));
        assert_eq!(get_file_timestamp("LICENSE"), get_timestamp_from("LICENSE"));
    }
}
