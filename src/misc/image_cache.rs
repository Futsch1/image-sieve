use std::{
    sync::Arc,
    sync::{mpsc, Mutex},
    thread,
};

use super::lru_map::LruMap;
use crate::item_sort_list::FileItem;
use crate::misc::images::ImageBuffer;
use sixtyfps::{Image, Rgb8Pixel, SharedPixelBuffer};

type ImagesMutex = Mutex<LruMap<ImageBuffer, String, 64>>;
type PrefetchCallback = Box<dyn Fn(ImageBuffer) + Send + 'static>;

pub struct ImageCache {
    images: Arc<ImagesMutex>,
    unknown_image: Image,
    sender: mpsc::Sender<(FileItem, u32, u32, Option<PrefetchCallback>)>,
    max_width: u32,
    max_height: u32,
}

impl ImageCache {
    pub fn new() -> Self {
        let images = LruMap::new();
        let mutex = Arc::new(Mutex::new(images));
        let mut pixel_buffer = SharedPixelBuffer::<Rgb8Pixel>::new(320, 200);
        crate::misc::images::draw_image(pixel_buffer.width(), pixel_buffer.make_mut_slice());
        let mutex_t = mutex.clone();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || prefetch_thread(mutex_t, rx));

        Self {
            images: mutex,
            unknown_image: Image::from_rgb8(pixel_buffer),
            sender: tx,
            max_width: 0,
            max_height: 0,
        }
    }

    pub fn restrict_size(&mut self, max_width: u32, max_height: u32) {
        if max_width > self.max_width || max_height > self.max_height {
            self.images.lock().unwrap().clear();
            self.max_width = max_width;
            self.max_height = max_height;
        }
    }

    pub fn load(&self, item: &FileItem) -> Image {
        if item.is_image() {
            let item_path = item.get_path().to_str().unwrap();
            let mut map = self.images.lock().unwrap();
            match map.get(String::from(item_path)) {
                Some(image) => crate::misc::images::get_sixtyfps_image(image),
                None => {
                    let image = crate::misc::images::get_image_buffer(
                        item,
                        self.max_width,
                        self.max_height,
                    );
                    let sixtyfps_image = crate::misc::images::get_sixtyfps_image(&image);
                    map.put(String::from(item_path), image);
                    sixtyfps_image
                }
            }
        } else {
            self.unknown_image.clone()
        }
    }

    pub fn prefetch(&self, item: &FileItem, done_callback: Option<PrefetchCallback>) {
        self.sender
            .send((item.clone(), self.max_width, self.max_height, done_callback))
            .ok();
    }
}

fn prefetch_thread(
    mutex: Arc<ImagesMutex>,
    receiver: mpsc::Receiver<(FileItem, u32, u32, Option<PrefetchCallback>)>,
) {
    for (prefetch_item, max_width, max_height, callback) in receiver {
        let item_path = prefetch_item.get_path().to_str().unwrap();
        let mut map = mutex.lock().unwrap();
        let key = String::from(item_path);
        let image = map.get(key);
        let image = if let Some(image) = image {
            image.clone()
        } else {
            let image =
                crate::misc::images::get_image_buffer(&prefetch_item, max_width, max_height);
            map.put(String::from(item_path), image.clone());
            image
        };
        if let Some(callback) = callback {
            callback(image);
        }
    }
}
