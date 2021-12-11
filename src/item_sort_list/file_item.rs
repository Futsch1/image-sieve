use std::cmp::Ordering;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::ops::Range;
use std::path::Path;
use std::path::PathBuf;

use img_hash::ImageHash;
use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;

use super::item_traits::Orientation;
use super::item_traits::PropertyResolver;

pub type HashType = ImageHash<Vec<u8>>;

/// A single file item with all properties required by image_sieve
#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
    /// Actual file path
    pub path: PathBuf,
    /// Time stamp of file creation (either from EXIF or from file system)
    timestamp: i64,
    /// Flag indicating if the file shall be taken over during sieving (true) or be discarded (false)
    take_over: bool,
    /// List of similar items as indices in the list of file items
    similar: Vec<usize>,
    /// Orientation of the image
    orientation: Option<Orientation>,
    /// Hash of the image
    #[serde(serialize_with = "serialize_hash")]
    #[serde(deserialize_with = "deserialize_hash")]
    hash: Option<HashType>,
    /// Image extension (implicit property of path, but saved for performance purposes)
    extension: String,
}

pub fn serialize_hash<S>(hash: &Option<HashType>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match hash {
        Some(hash) => hash.to_base64().serialize(s),
        None => "".serialize(s),
    }
}

pub fn deserialize_hash<'de, D>(deserializer: D) -> Result<Option<HashType>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let hash_str = String::deserialize(deserializer)?;
    if hash_str.is_empty() {
        Ok(None)
    } else {
        Ok(Some(HashType::from_base64(&hash_str).unwrap()))
    }
}

impl FileItem {
    /// Create a new file item from a path and initialize properties from serialization
    pub fn new(
        path: PathBuf,
        property_resolver: Box<dyn PropertyResolver>,
        take_over: bool,
        encoded_hash: &str,
    ) -> Self {
        let timestamp = property_resolver.get_timestamp();
        let orientation = property_resolver.get_orientation();
        let hash = process_encoded_hash(encoded_hash);
        let extension = extension(&path);

        Self {
            path,
            timestamp,
            take_over,
            similar: Vec::new(),
            orientation,
            hash,
            extension,
        }
    }

    /// Construct a dummy/empty file item
    #[cfg(test)]
    pub fn dummy(path: &str, timestamp: i64, take_over: bool) -> Self {
        Self {
            path: PathBuf::from(path),
            timestamp,
            orientation: None,
            take_over,
            similar: Vec::new(),
            hash: None,
            extension: extension(Path::new(path)),
        }
    }

    /// Set the take over property to make a file item be discarded or taken over in the sieving process
    pub fn set_take_over(&mut self, take_over: bool) {
        self.take_over = take_over;
    }

    /// Get the take over property
    pub fn get_take_over(&self) -> bool {
        self.take_over
    }

    /// Get the time stamp of the file item
    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Get the time stamp of the file item formatted as string
    fn get_date_str(&self) -> String {
        chrono::NaiveDateTime::from_timestamp(self.timestamp, 0)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
    }

    /// Get the size of a file item in bytes
    pub fn get_size(&self) -> u64 {
        let result = self.path.metadata();
        match result {
            Ok(metadata) => metadata.len(),
            Err(_) => 0,
        }
    }

    /// Adds another item's index as a similar item
    pub fn add_similar(&mut self, other_index: usize) {
        self.similar.push(other_index);
    }

    /// Add a list of similars
    pub fn add_similars(&mut self, similars: &Range<usize>) {
        self.similar.extend(similars.clone().into_iter());
    }

    /// Get the list of similar item indices. Needs a mut because the cleanup of the similar list is done
    /// in a lazy fashion and might happen here.
    pub fn get_similars(&self) -> &Vec<usize> {
        &self.similar
    }

    /// Reset the list of similar item indices
    pub fn reset_similars(&mut self) {
        self.similar.clear()
    }

    fn has_similars(&self) -> bool {
        return self.similar.is_empty();
    }

    pub fn clean_similars(&mut self, item_index: usize) {
        self.similar.sort();
        self.similar.dedup();
        if let Ok(similar_index) = self.similar.binary_search(&item_index) {
            self.similar.remove(similar_index);
        }
    }

    /// Get the orientation of the image
    pub fn get_orientation(&self) -> Option<&Orientation> {
        self.orientation.as_ref()
    }

    /// Gets a string representing the item type and if it has simlar items or not, if it will be discarded and the item path
    pub fn get_item_string(&self, base_path: &Path) -> String {
        let path = self.path.strip_prefix(base_path).unwrap_or(&self.path);
        let similars_str = if !self.has_similars() {
            "\u{1F500} "
        } else {
            ""
        };
        let extension_str = self.extension_to_unicode_icon();
        let take_over_str = if self.take_over { "" } else { "\u{1F5D1} " };
        let strings = [
            similars_str,
            extension_str,
            take_over_str,
            path.to_str().unwrap(),
        ];
        strings.join(" ")
    }

    /// Check if the item is an image
    pub fn is_image(&self) -> bool {
        matches!(
            self.path
                .extension()
                .unwrap()
                .to_ascii_lowercase()
                .to_str()
                .unwrap(),
            "jpg" | "png" | "tif"
        )
    }

    /// Check if the item is a video
    pub fn is_video(&self) -> bool {
        matches!(
            self.path.extension().unwrap().to_str().unwrap(),
            "mp4" | "avi" | "mts"
        )
    }

    /// Get a list of allowed extensions
    pub fn get_extensions() -> [&'static str; 6] {
        ["jpg", "png", "tif", "mp4", "avi", "mts"]
    }

    /// Get the unicode icon for the extension
    fn extension_to_unicode_icon(&self) -> &str {
        if self.is_image() {
            "\u{1F4F7} "
        } else if self.is_video() {
            "\u{1F4F9} "
        } else {
            ""
        }
    }

    /// Set the image hash
    pub fn set_hash(&mut self, hash: ImageHash<Vec<u8>>) {
        self.hash = Some(hash);
    }

    /// Set the image hash from an encoded hash
    pub fn set_encoded_hash(&mut self, encoded_hash: &str) {
        self.hash = process_encoded_hash(encoded_hash);
    }

    /// Gets the image hash as an encoded hash
    pub fn get_encoded_hash(&self) -> String {
        if let Some(hash) = self.hash.clone() {
            hash.to_base64()
        } else {
            String::new()
        }
    }

    /// Check if the file item has a hash
    pub fn has_hash(&self) -> bool {
        self.hash.is_some()
    }

    /// Get the image hash distance to another file item
    pub fn get_hash_distance(&self, other: &FileItem) -> u32 {
        if self.has_hash() && other.has_hash() {
            self.hash
                .as_ref()
                .unwrap()
                .dist(other.hash.as_ref().unwrap())
        } else {
            u32::MAX
        }
    }
}

impl Display for FileItem {
    /// Gets the item text, composed of the item string, the item size in KB, the item date and an optional event
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let item_text = self.get_item_string(Path::new(""));
        let item_size = self.get_size() / 1024;
        let item_date = self.get_date_str();
        write!(f, "{} - {}, {} KB", item_text, item_date, item_size)
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

/// Gets the extension from a path
fn extension(path: &Path) -> String {
    let extension = path.extension().unwrap_or_default().to_ascii_lowercase();
    String::from(extension.to_str().unwrap())
}

/// Process an encoded hash and create an image hash from it
fn process_encoded_hash(encoded_hash: &str) -> Option<ImageHash<Vec<u8>>> {
    if !encoded_hash.is_empty() {
        if let Ok(hash) = ImageHash::from_base64(encoded_hash) {
            Some(hash)
        } else {
            None
        }
    } else {
        None
    }
}
