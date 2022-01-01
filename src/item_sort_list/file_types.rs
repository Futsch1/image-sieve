use std::path::Path;

pub fn is_image(path: &Path) -> bool {
    matches!(
        path.extension()
            .unwrap()
            .to_ascii_lowercase()
            .to_str()
            .unwrap(),
        "jpg" | "png" | "tif"
    )
}

pub fn is_video(path: &Path) -> bool {
    matches!(
        path.extension().unwrap().to_str().unwrap(),
        "mp4" | "avi" | "mts" | "mov"
    )
}

/// Get a list of allowed extensions
pub fn get_extensions() -> [&'static str; 7] {
    ["jpg", "png", "tif", "mp4", "avi", "mts", "mov"]
}
