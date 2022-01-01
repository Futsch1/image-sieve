use std::path::Path;

pub fn is_image(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        matches!(
            extension.to_ascii_lowercase().to_str().unwrap(),
            "jpg" | "png" | "tif"
        )
    } else {
        false
    }
}

pub fn is_video(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        matches!(
            extension.to_ascii_lowercase().to_str().unwrap(),
            "mp4" | "avi" | "mts" | "mov"
        )
    } else {
        false
    }
}

/// Get a list of allowed extensions
pub fn get_extensions() -> [&'static str; 7] {
    ["jpg", "png", "tif", "mp4", "avi", "mts", "mov"]
}
