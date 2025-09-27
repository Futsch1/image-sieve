use std::path::Path;

const IMAGE: &[&str] = &[
    "jpg", "png", "tif", "jpeg", "jpe", "gif", "bmp", "webp", "tiff", "jxl",
];

const RAW: &[&str] = &[
    "mrw", "arw", "srf", "sr2", "mef", "orf", "srw", "erf", "kdc", "dcs", "rw2", "raf", "dcr",
    "dng", "pef", "crw", "raw", "iiq", "3fr", "nrw", "nef", "mos", "cr2", "ari",
];

const VIDEO: &[&str] = &[
    "mp4", "mp4v", "mpeg4", "avi", "mts", "mov", "mpeg", "mpg", "mjpeg", "mjpg", "mjp", "mp2v",
];

const HEIF: &[&str] = &[
    "heic", "heif"
];

pub fn is_image(path: &Path) -> bool {
    is_extension_in(path, IMAGE)
}

pub fn is_raw_image(path: &Path) -> bool {
    is_extension_in(path, RAW)
}

pub fn is_heif_image(path: &Path) -> bool {
    is_extension_in(path, HEIF)
}

pub fn is_video(path: &Path) -> bool {
    is_extension_in(path, VIDEO)
}

pub fn is_any(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        let extension = extension.to_ascii_lowercase();
        let extension = &extension.to_str().unwrap();
        IMAGE.contains(extension) || VIDEO.contains(extension) || RAW.contains(extension) || HEIF.contains(extension)
    } else {
        false
    }
}

fn is_extension_in(path: &Path, extensions: &[&str]) -> bool {
    if let Some(extension) = path.extension() {
        extensions.contains(&extension.to_ascii_lowercase().to_str().unwrap())
    } else {
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_extensions() {
        assert!(is_image(Path::new("/path/to/image.jpg")));
        assert!(is_image(Path::new("/path/to/image.PNG")));
        assert!(!is_image(Path::new("/path/to/image")));

        assert!(is_raw_image(Path::new("/path/to/image.mrw")));
        assert!(is_raw_image(Path::new("/path/to/image.CR2")));
        assert!(!is_raw_image(Path::new("/path/to/image.zip")));

        assert!(is_video(Path::new("path/to/video.mpeg")));
        assert!(is_video(Path::new("path/to/video.AVI")));
        assert!(!is_video(Path::new("path/to/video.jpg")));

        assert!(is_heif_image(Path::new("/path/to/image.heic")));
        assert!(is_heif_image(Path::new("/path/to/image.HEIF")));
        assert!(!is_heif_image(Path::new("/path/to/image.png")));

        assert!(is_any(Path::new("/path/to/image.jpg")));
        assert!(is_any(Path::new("/path/to/image.CR2")));
        assert!(is_any(Path::new("/path/to/video.mov")));
        assert!(!is_any(Path::new("/path/to/video.zip")));
    }
}
