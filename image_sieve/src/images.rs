extern crate image;
extern crate sixtyfps;

use item_sort_list::FileItem;

pub type ImageBuffer = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

pub fn get_image_buffer(item: &FileItem) -> ImageBuffer {
    let path = item.get_path();
    let rotation = match item.get_orientation() {
        Some(orientation) => match orientation {
            item_sort_list::Orientation::Landscape => 0,
            item_sort_list::Orientation::Portrait90 => 90,
            item_sort_list::Orientation::Landscape180 => 180,
            item_sort_list::Orientation::Portrait270 => 270,
        },
        None => 0,
    };
    load_image_and_rotate(path, rotation)
}

pub fn get_empty_image() -> sixtyfps::Image {
    let buffer = sixtyfps::SharedPixelBuffer::new(1, 1);
    sixtyfps::Image::from_rgba8(buffer)
}

pub fn get_sixtyfps_image(buffer: &ImageBuffer) -> sixtyfps::Image {
    let buffer = sixtyfps::SharedPixelBuffer::<sixtyfps::Rgba8Pixel>::clone_from_slice(
        buffer.as_raw(),
        buffer.width() as _,
        buffer.height() as _,
    );
    sixtyfps::Image::from_rgba8(buffer)
}

fn load_image_and_rotate(path: &std::path::Path, rotate: i32) -> ImageBuffer {
    let mut cat_image = image::open(path).expect("Error loading image").into_rgba8();

    let image = match rotate {
        90 => image::imageops::rotate90(&mut cat_image),
        180 => image::imageops::rotate180(&mut cat_image),
        270 => image::imageops::rotate270(&mut cat_image),
        _ => cat_image,
    };
    image
}

/// Draw a greyish image from a pixel buffer
pub fn draw_image(width: usize, buffer: &mut [sixtyfps::Rgb8Pixel]) {
    let mut t: bool = false;
    for (i, p) in buffer.iter_mut().enumerate() {
        if i % width == 0 {
            t = (i / width) % 2 == 0;
        }
        let val: u8 = if t { 0x66 } else { 0xFF };
        p.r = val;
        p.g = val;
        p.b = val;
        t = !t;
    }
}
