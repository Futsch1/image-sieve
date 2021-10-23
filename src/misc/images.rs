extern crate image;
extern crate sixtyfps;

use crate::item_sort_list::FileItem;

pub type ImageBuffer = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

pub fn get_image_buffer(item: &FileItem) -> ImageBuffer {
    let path = item.get_path();
    let rotation = match item.get_orientation() {
        Some(orientation) => match orientation {
            crate::item_sort_list::Orientation::Landscape => 0,
            crate::item_sort_list::Orientation::Portrait90 => 90,
            crate::item_sort_list::Orientation::Landscape180 => 180,
            crate::item_sort_list::Orientation::Portrait270 => 270,
        },
        None => 0,
    };
    load_image_and_rotate(path, rotation).unwrap_or_else(|_| ImageBuffer::new(1, 1))
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

fn load_image_and_rotate(
    path: &std::path::Path,
    rotate: i32,
) -> Result<ImageBuffer, image::ImageError> {
    let cat_image = image::open(path)?.into_rgba8();

    Ok(match rotate {
        90 => image::imageops::rotate90(&cat_image),
        180 => image::imageops::rotate180(&cat_image),
        270 => image::imageops::rotate270(&cat_image),
        _ => cat_image,
    })
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