use std::path::Path;

const IMAGE: &[&str] = &[
    "jpg", "png", "tif", "jpeg", "jpe", "gif", "bmp", "webp", "tiff",
];

const RAW: &[&str] = &[
    "mrw", "arw", "srf", "sr2", "mef", "orf", "srw", "erf", "kdc", "dcs", "rw2", "raf", "dcr",
    "dng", "pef", "crw", "raw", "iiq", "3fr", "nrw", "nef", "mos", "cr2", "ari",
];

const VIDEO: &[&str] = &["mp4", "avi", "mts", "mov"];

pub fn is_image(path: &Path) -> bool {
    is_extension_in(path, IMAGE)
}

pub fn is_raw_image(path: &Path) -> bool {
    is_extension_in(path, RAW)
}

pub fn is_video(path: &Path) -> bool {
    is_extension_in(path, VIDEO)
}

pub fn is_any(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        let extension = extension.to_ascii_lowercase();
        let extension = &extension.to_str().unwrap();
        IMAGE.contains(extension) || VIDEO.contains(extension) || RAW.contains(extension)
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
