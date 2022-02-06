use std::{
    collections::VecDeque,
    sync::Arc,
    sync::{mpsc, Mutex},
    thread,
};

use super::lru_map::LruMap;
use crate::item_sort_list::FileItem;
use crate::misc::images::ImageBuffer;
use sixtyfps::Image;

/// The least recently used map used to store the images protected by a mutex.
type ImagesMapMutex = Mutex<LruMap<ImageBuffer, String, 64>>;
/// The queue with images to load protected by a mutex.
type LoadQueue = Mutex<VecDeque<LoadImageCommand>>;
/// The callback which is executed when an image was loaded (is no sixtyfps::Image because that is not "Send")
pub type DoneCallback = Box<dyn Fn(ImageBuffer) + Send + 'static>;

const HOURGLASS_PNG: &[u8; 5533] = include_bytes!("hourglass.png");

/// Purpose of the image to load from the cache
pub enum Purpose {
    /// The image is the currently selected image and needs to be loaded as soon as possible
    CurrentImage,
    /// This image is an image in the similar list and needs to be loaded soon, but not immediately
    SimilarImage,
    /// The image is one of the next in the list and should be loaded to increase the perceived speed, but it is not urgent
    Prefetch,
}

struct LoadImageCommand {
    pub file_item: FileItem,
    pub width: u32,
    pub height: u32,
    pub callback: Option<DoneCallback>,
}

impl PartialEq for LoadImageCommand {
    fn eq(&self, other: &Self) -> bool {
        self.file_item == other.file_item
    }
}

/// An image cache that provides some priorization on the images to load. The cache loads images in the background and executes
/// a callback when the image is loaded.
/// The cache can restrict the sizes of loaded images to reduce memory usage.
/// The cache implements two separate threads for loading to implement the priorization. Selected images are loaded from one thread,
/// the other thread loads the similar images and the prefetch images. In order to priorize the similar images, these commands are
/// added to the front of the load queue, while the prefetch image commands are added to the back.
pub struct ImageCache {
    /// Map with the images
    images: Arc<ImagesMapMutex>,
    /// Buffered image to be displayed while waiting for an image to load
    waiting_image: Image,
    /// Maximum width of the images to load
    max_width: u32,
    /// Maximum height of the images to load
    max_height: u32,
    /// Queue of load commands for the primary load thread
    primary_queue: Arc<LoadQueue>,
    /// Sender to the primary load thread
    primary_sender: mpsc::Sender<()>,
    /// Queue of load commands for the secondary load thread
    secondary_queue: Arc<LoadQueue>,
    /// Sender to the secondary load thread
    secondary_sender: mpsc::Sender<()>,
}

impl ImageCache {
    /// Create a new image cache
    pub fn new() -> Self {
        let images = LruMap::new();
        let mutex = Arc::new(Mutex::new(images));

        let mutex_t = mutex.clone();
        let (primary_sender, rx) = mpsc::channel();
        let primary_queue = Arc::new(LoadQueue::new(VecDeque::new()));
        let queue_t = primary_queue.clone();
        thread::spawn(move || load_image_thread(mutex_t, queue_t, rx));

        let mutex_t = mutex.clone();
        let (secondary_sender, rx) = mpsc::channel();
        let secondary_queue = Arc::new(LoadQueue::new(VecDeque::new()));
        let queue_t = secondary_queue.clone();
        thread::spawn(move || load_image_thread(mutex_t, queue_t, rx));

        Self {
            images: mutex,
            waiting_image: ImageCache::get_hourglass(),
            max_width: 0,
            max_height: 0,
            primary_queue,
            primary_sender,
            secondary_queue,
            secondary_sender,
        }
    }

    /// Gets the hourglass image to indicate waiting
    /// The image is compiled into the binary
    fn get_hourglass() -> Image {
        crate::misc::images::get_sixtyfps_image(
            &crate::misc::images::image_from_buffer(HOURGLASS_PNG).unwrap(),
        )
    }

    /// Purge all running commands
    pub fn purge(&self) {
        self.primary_queue.lock().unwrap().clear();
        self.secondary_queue.lock().unwrap().clear();
    }

    /// Sets the maximum width and height of the images to load
    pub fn restrict_size(&mut self, max_width: u32, max_height: u32) {
        if max_width > self.max_width || max_height > self.max_height {
            self.images.lock().unwrap().clear();
            self.max_width = max_width;
            self.max_height = max_height;
        }
    }

    /// Gets an image from the cache
    pub fn get(&self, item: &FileItem) -> Option<Image> {
        let item_path = item.path.to_str().unwrap();
        let mut map = self.images.lock().unwrap();
        map.get(String::from(item_path))
            .map(|image| crate::misc::images::get_sixtyfps_image(image))
    }

    /// Gets the waiting image
    pub fn get_waiting(&self) -> Image {
        self.waiting_image.clone()
    }

    /// Loads an image from the cache
    /// The purpose of the image needs to be indicated to determine the loading priority. When the image was loaded,
    /// the done callback is executed.
    pub fn load(&self, item: &FileItem, purpose: Purpose, done_callback: Option<DoneCallback>) {
        let command = LoadImageCommand {
            file_item: item.clone(),
            width: self.max_width,
            height: self.max_height,
            callback: done_callback,
        };
        match purpose {
            Purpose::CurrentImage => {
                let mut queue = self.primary_queue.lock().unwrap();
                queue.clear();
                queue.push_front(command);
                self.primary_sender.send(()).ok();
            }
            Purpose::SimilarImage => {
                let mut queue = self.secondary_queue.lock().unwrap();
                queue.push_back(command);
                self.secondary_sender.send(()).ok();
            }
            Purpose::Prefetch => {
                let mut queue = self.secondary_queue.lock().unwrap();
                if !queue.contains(&command) {
                    queue.push_back(command);
                }
                self.secondary_sender.send(()).ok();
            }
        }
    }
}

/// Loads images in the background after receiving a trigger message. The message sent to the thread is empty, the actual
/// commands are contained in the load queue.
fn load_image_thread(
    cache: Arc<ImagesMapMutex>,
    load_queue: Arc<LoadQueue>,
    receiver: mpsc::Receiver<()>,
) {
    for () in receiver {
        let next_item = load_queue.lock().unwrap().pop_front();
        if next_item.is_none() {
            continue;
        }
        let command = next_item.unwrap();
        let item_path = command.file_item.path.to_str().unwrap();
        // First try to get the image from the cache
        let contains_key = {
            let map = cache.lock().unwrap();
            map.contains(String::from(item_path))
        };
        // If it is not in the cache, load it from the file and put it into the cache
        if !contains_key {
            let image_buffer = if command.file_item.is_video() {
                crate::misc::video_to_image::get_image_buffer(
                    &command.file_item,
                    command.width,
                    command.height,
                )
            } else {
                crate::misc::images::get_image_buffer(
                    &command.file_item,
                    command.width,
                    command.height,
                )
            };
            let mut map = cache.lock().unwrap();
            map.put(String::from(item_path), image_buffer.clone());
        }

        // If a callback was indicated, execute it passing a clone of the image
        if let Some(callback) = command.callback {
            let image = {
                let mut map = cache.lock().unwrap();
                map.get(String::from(item_path)).cloned()
            }
            .unwrap();
            callback(image);
        }
    }
}
