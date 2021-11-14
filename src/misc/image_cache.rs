use std::{collections::VecDeque, sync::Arc, sync::{mpsc, Mutex}, thread};

use super::{lru_map::LruMap};
use crate::item_sort_list::FileItem;
use crate::misc::images::ImageBuffer;
use sixtyfps::{Image, Rgb8Pixel, SharedPixelBuffer};

type ImagesMutex = Mutex<LruMap<ImageBuffer, String, 64>>;
type LoadQueue = Mutex<VecDeque<LoadImageCommand>>;
pub type PrefetchCallback = Box<dyn Fn(ImageBuffer) + Send + 'static>;
pub type LoadImageCommand = (FileItem, u32, u32, Option<PrefetchCallback>);

pub enum Purpose {
    SelectedImage,
    SimilarImage,
    Prefetch
}

pub struct ImageCache {
    images: Arc<ImagesMutex>,
    unknown_image: Image,
    waiting_image: Image,
    max_width: u32,
    max_height: u32,
    primary_queue: Arc<LoadQueue>,
    primary_sender: mpsc::Sender<()>,
    secondary_queue: Arc<LoadQueue>,
    secondary_sender: mpsc::Sender<()>
}

impl ImageCache {
    pub fn new() -> Self {
        let images = LruMap::new();
        let mutex = Arc::new(Mutex::new(images));
        let mut pixel_buffer = SharedPixelBuffer::<Rgb8Pixel>::new(320, 200);
        crate::misc::images::draw_image(pixel_buffer.width(), pixel_buffer.make_mut_slice());

        let mutex_t = mutex.clone();
        let (primary_sender, rx) = mpsc::channel();
        let primary_queue = Arc::new(LoadQueue::new(VecDeque::new()));
        let queue_t = primary_queue.clone();
        thread::spawn(move || prefetch_thread(mutex_t, queue_t, rx));

        let mutex_t = mutex.clone();
        let (secondary_sender, rx) = mpsc::channel();
        let secondary_queue = Arc::new(LoadQueue::new(VecDeque::new()));
        let queue_t = secondary_queue.clone();
        thread::spawn(move || prefetch_thread(mutex_t, queue_t, rx));

        Self {
            images: mutex,
            unknown_image: Image::from_rgb8(pixel_buffer),
            waiting_image: ImageCache::get_hourglass(),
            max_width: 0,
            max_height: 0,
            primary_queue,
            primary_sender,
            secondary_queue,
            secondary_sender
        }
    }

    fn get_hourglass() -> Image {
        let bytes = include_bytes!("hourglass.png");
        crate::misc::images::get_sixtyfps_image(
            &crate::misc::images::image_from_buffer(bytes).unwrap(),
        )
    }

    pub fn restrict_size(&mut self, max_width: u32, max_height: u32) {
        if max_width > self.max_width || max_height > self.max_height {
            self.images.lock().unwrap().clear();
            self.max_width = max_width;
            self.max_height = max_height;
        }
    }

    pub fn get(&self, item: &FileItem) -> Option<Image> {
        if item.is_image() {
            let item_path = item.get_path().to_str().unwrap();
            let mut map = self.images.lock().unwrap();
            map.get(String::from(item_path))
                .map(|image| crate::misc::images::get_sixtyfps_image(image))
        } else {
            Some(self.get_unknown())
        }
    }

    pub fn get_unknown(&self) -> Image {
        self.unknown_image.clone()
    }

    pub fn get_waiting(&self) -> Image {
        self.waiting_image.clone()
    }

    pub fn load(&self, item: &FileItem, purpose: Purpose, done_callback: Option<PrefetchCallback>) {
        let command = (item.clone(), self.max_width, self.max_width, done_callback);
        match purpose
        {
            Purpose::SelectedImage => {
                let mut queue = self.primary_queue.lock().unwrap();
                queue.clear();
                queue.push_front(command);
                self.primary_sender.send(()).ok();
            },
            Purpose::SimilarImage => {
                let mut queue = self.secondary_queue.lock().unwrap();
                queue.push_front(command);
                self.secondary_sender.send(()).ok();
            },
            Purpose::Prefetch => {
                let mut queue = self.secondary_queue.lock().unwrap();
                queue.push_back(command);
                self.secondary_sender.send(()).ok();
            }
        }
    }
}

fn prefetch_thread(
    cache: Arc<ImagesMutex>,
    load_queue: Arc<LoadQueue>,
    receiver: mpsc::Receiver<()>,
) {
    for () in receiver {
        let next_item = load_queue.lock().unwrap().pop_front();
        if next_item.is_none() {
            continue;
        }
        let (prefetch_item, max_width, max_height, callback) = next_item.unwrap();
        let item_path = prefetch_item.get_path().to_str().unwrap();
        // First try to get from cache
        let contains_key = {
            let map = cache.lock().unwrap();
            map.contains(String::from(item_path))
        };
        if !contains_key {
            let image_buffer =
                crate::misc::images::get_image_buffer(&prefetch_item, max_width, max_height);
            let mut map = cache.lock().unwrap();
            map.put(String::from(item_path), image_buffer.clone());
            // println!("Loaded {}", item_path);
        }

        if let Some(callback) = callback {
            let image = {
                let mut map = cache.lock().unwrap();
                map.get(String::from(item_path)).cloned()
            }
            .unwrap();
            callback(image);
        }
    }
}
