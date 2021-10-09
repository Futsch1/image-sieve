use std::{
    sync::Arc,
    sync::{mpsc, Mutex},
    thread,
};

use crate::lru_map::LruMap;
use item_sort_list::FileItem;
use sixtyfps::{Image, Rgb8Pixel, SharedPixelBuffer};

type ImagesMutex = Mutex<LruMap<crate::images::ImageBuffer, String, 16>>;

pub struct ImageCache {
    images: Arc<ImagesMutex>,
    unknown_image: Image,
    sender: mpsc::Sender<FileItem>,
}

impl ImageCache {
    pub fn new() -> Self {
        let images = LruMap::new();
        let mutex = Arc::new(Mutex::new(images));
        let mut pixel_buffer = SharedPixelBuffer::<Rgb8Pixel>::new(320, 200);
        crate::images::draw_image(pixel_buffer.width(), pixel_buffer.make_mut_slice());
        let mutex_t = mutex.clone();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || prefetch_thread(mutex_t, rx));

        Self {
            images: mutex,
            unknown_image: Image::from_rgb8(pixel_buffer),
            sender: tx,
        }
    }

    pub fn load(&self, item: &FileItem) -> Image {
        if item.is_image() {
            let item_path = item.get_path().to_str().unwrap();
            let mut map = self.images.lock().unwrap();
            match map.get(String::from(item_path)) {
                Some(image) => crate::images::get_sixtyfps_image(&image),
                None => {
                    let image = crate::images::get_image_buffer(item);
                    let sixtyfps_image = crate::images::get_sixtyfps_image(&image);
                    map.put(String::from(item_path), image);
                    sixtyfps_image
                }
            }
        } else {
            self.unknown_image.clone()
        }
    }

    pub fn prefetch(&self, item: &FileItem) {
        self.sender.send(item.clone()).ok();
    }
}

fn prefetch_thread(mutex: Arc<ImagesMutex>, receiver: mpsc::Receiver<FileItem>) {
    for prefetch_item in receiver {
        let item_path = prefetch_item.get_path().to_str().unwrap();
        let mut map = mutex.lock().unwrap();
        if map.get(String::from(item_path)).is_none() {
            let image = crate::images::get_image_buffer(&prefetch_item);
            map.put(String::from(item_path), image);
        }
    }
}
