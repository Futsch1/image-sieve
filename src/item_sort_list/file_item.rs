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

use super::Format;
use super::file_types::is_image;
use super::file_types::is_raw_image;
use super::file_types::is_video;
use super::item_traits::Orientation;
use super::item_traits::PropertyResolver;
use super::timestamp_to_string;

pub type HashType = ImageHash<Vec<u8>>;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
enum ItemType {
    Image,
    Video,
    RawImage,
}

/// A single file item with all properties required by image_sieve
#[derive(Eq, Debug, Clone, Serialize, Deserialize)]
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
    /// File item type
    item_type: Option<ItemType>,
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

fn get_item_type(path: &Path) -> ItemType {
    match (is_image(path), is_video(path), is_raw_image(path)) {
        (true, _, _) => ItemType::Image,
        (_, true, _) => ItemType::Video,
        (_, _, true) => ItemType::RawImage,
        _ => panic!("FileItem::new: File type not supported"),
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
        let item_type = get_item_type(&path);

        Self {
            path,
            timestamp,
            take_over,
            similar: Vec::new(),
            orientation,
            hash,
            item_type: Some(item_type),
        }
    }

    /// Construct a dummy/empty file item
    #[cfg(test)]
    pub fn dummy(path: &str, timestamp: i64, take_over: bool) -> Self {
        let path = PathBuf::from(path);
        let item_type = get_item_type(&path);
        Self {
            path,
            timestamp,
            orientation: Some(Orientation::Landscape),
            take_over,
            similar: Vec::new(),
            hash: None,
            item_type: Some(item_type),
        }
    }

    /// Called after deserialization to setup all option fields
    pub fn deserialized(&mut self) {
        if self.item_type.is_none() {
            self.item_type = Some(get_item_type(&self.path));
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
        timestamp_to_string(self.timestamp, Format::DateTime)
    }

    /// Get the size of a file item in bytes
    pub fn get_size(&self) -> u64 {
        let result = self.path.metadata();
        match result {
            Ok(metadata) => metadata.len(),
            Err(_) => 0,
        }
    }

    /// Adds a vector of similars
    pub fn add_similar_vec(&mut self, similars: &[usize]) {
        self.similar.extend(similars);
    }

    /// Add a range of similars
    pub fn add_similar_range(&mut self, similars: &Range<usize>) {
        self.similar.extend(similars.clone());
    }

    /// Get the list of similar item indices.
    pub fn get_similars(&self) -> &Vec<usize> {
        &self.similar
    }

    /// Reset the list of similar item indices
    pub fn reset_similars(&mut self) {
        self.similar.clear()
    }

    fn has_similars(&self) -> bool {
        self.similar.is_empty()
    }

    pub fn clean_similars(&mut self, item_index: usize) {
        self.similar.sort_unstable();
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
        let similars_str = if !self.has_similars() { "🔀" } else { "" };
        let extension_str = self.extension_to_unicode_icon();
        let take_over_str = if self.take_over { "" } else { "🗑" };
        let strings: Vec<&str> = [
            similars_str,
            extension_str,
            take_over_str,
            path.to_str().unwrap(),
        ]
        .iter()
        .filter(|&s| !s.is_empty())
        .copied()
        .collect();
        strings.join(" ")
    }

    /// Check if the item is an image
    pub fn is_image(&self) -> bool {
        *self.item_type.as_ref().unwrap() == ItemType::Image
    }

    /// Check if the item is a raw image
    pub fn is_raw_image(&self) -> bool {
        *self.item_type.as_ref().unwrap() == ItemType::RawImage
    }

    /// Check if the item is a video
    pub fn is_video(&self) -> bool {
        *self.item_type.as_ref().unwrap() == ItemType::Video
    }

    /// Get the unicode icon for the extension
    fn extension_to_unicode_icon(&self) -> &str {
        if self.is_image() || self.is_raw_image() {
            "📷"
        } else if self.is_video() {
            "📹"
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

impl PartialEq for FileItem {
    /// Check if two file items are equal
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
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

/// Process an encoded hash and create an image hash from it
fn process_encoded_hash(encoded_hash: &str) -> Option<ImageHash<Vec<u8>>> {
    if !encoded_hash.is_empty()
        && let Ok(hash) = ImageHash::from_base64(encoded_hash)
    {
        Some(hash)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item_sort_list::{Orientation, item_traits::PropertyResolver};

    struct MockResolver {
        timestamp: i64,
        orientation: Option<Orientation>,
    }

    impl MockResolver {
        fn new(timestamp: i64, orientation: Option<Orientation>) -> Self {
            MockResolver {
                timestamp,
                orientation,
            }
        }
    }

    impl PropertyResolver for MockResolver {
        fn get_timestamp(&self) -> i64 {
            self.timestamp
        }

        fn get_orientation(&self) -> Option<Orientation> {
            self.orientation.clone()
        }
    }

    #[test]
    fn test_new() {
        let resolver = Box::new(MockResolver::new(10, Some(Orientation::Landscape180)));
        let file_item = FileItem::new(PathBuf::from("tests/test.jpg"), resolver, true, "");

        assert_eq!(
            Some(&Orientation::Landscape180),
            file_item.get_orientation()
        );
        assert_eq!(10, file_item.get_timestamp());
        assert!(file_item.get_take_over());
        assert_eq!(7383, file_item.get_size());
        assert_eq!("", file_item.get_encoded_hash());

        let resolver = Box::new(MockResolver::new(10, Some(Orientation::Landscape180)));
        FileItem::new(PathBuf::from("tests/not_existing.jpg"), resolver, true, "");
    }

    #[test]
    #[should_panic]
    fn test_new_invalid() {
        let resolver = Box::new(MockResolver::new(10, Some(Orientation::Landscape180)));
        FileItem::new(PathBuf::from("tests/test"), resolver, true, "");
    }

    #[test]
    fn test_hashes() {
        let resolver = Box::new(MockResolver::new(10, Some(Orientation::Landscape180)));
        let mut file_item = FileItem::new(
            PathBuf::from("tests/test.jpg"),
            resolver,
            true,
            "Wrong_hash",
        );
        assert_eq!("", file_item.get_encoded_hash());
        assert!(!file_item.has_hash());

        let resolver = Box::new(MockResolver::new(10, Some(Orientation::Landscape180)));
        let hash = HashType::from_bytes(&[0x61, 0x62, 0x63])
            .unwrap()
            .to_base64();
        let mut file_item2 = FileItem::new(PathBuf::from("tests/test.jpg"), resolver, true, &hash);
        assert_eq!(hash, file_item2.get_encoded_hash());
        assert!(file_item2.has_hash());

        let hash = HashType::from_bytes(&[0x64, 0x65, 0x66, 0x67])
            .unwrap()
            .to_base64();
        file_item2.set_encoded_hash(&hash);
        assert_eq!(hash, file_item2.get_encoded_hash());
        assert!(file_item2.has_hash());

        file_item2.set_hash(HashType::from_bytes(&[0x64, 0x65, 0x66, 0x67]).unwrap());
        assert_eq!(hash, file_item2.get_encoded_hash());
        assert!(file_item2.has_hash());

        assert_eq!(file_item.get_hash_distance(&file_item2), u32::MAX);
        file_item.set_hash(HashType::from_bytes(&[0x64, 0x65, 0x66, 0x67]).unwrap());

        assert_eq!(
            file_item.get_hash_distance(&file_item2),
            file_item2.get_hash_distance(&file_item)
        );
        assert_eq!(file_item.get_hash_distance(&file_item2), 0);
    }

    #[test]
    fn test_takeover() {
        let resolver = Box::new(MockResolver::new(10, Some(Orientation::Landscape180)));
        let mut file_item = FileItem::new(
            PathBuf::from("tests/test.jpg"),
            resolver,
            true,
            "Wrong_hash",
        );

        file_item.set_take_over(false);
        assert!(!file_item.get_take_over());
    }
}
