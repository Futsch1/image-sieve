use std::{cmp::max, num::NonZeroU32};

use fast_image_resize::{
    DifferentTypesOfPixelsError, Image, ImageBufferError, MulDiv, MulDivImageError,
    MulDivImagesError, PixelType, ResizeAlg, Resizer,
};

use super::images::ImageBuffer;

/// We do not really care about the underlying error, so wrap all fast_image_resize errors to a single type
#[derive(Debug)]
pub enum ResizeImageError {
    Error,
}

impl From<ImageBufferError> for ResizeImageError {
    fn from(_: ImageBufferError) -> Self {
        ResizeImageError::Error
    }
}

impl From<MulDivImageError> for ResizeImageError {
    fn from(_: MulDivImageError) -> Self {
        ResizeImageError::Error
    }
}

impl From<MulDivImagesError> for ResizeImageError {
    fn from(_: MulDivImagesError) -> Self {
        ResizeImageError::Error
    }
}

impl From<DifferentTypesOfPixelsError> for ResizeImageError {
    fn from(_: DifferentTypesOfPixelsError) -> Self {
        ResizeImageError::Error
    }
}

/// Resize an image buffer with the nearest neighbor method
pub fn resize_image(
    mut src_image: ImageBuffer,
    new_width: u32,
    new_height: u32,
) -> Result<ImageBuffer, ResizeImageError> {
    let width = src_image.width();
    let height = src_image.height();
    let src_image_data = Image::from_slice_u8(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        &mut src_image,
        PixelType::U8x4,
    )?;
    let src_view = src_image_data.view();
    let mut premultiplied_src_image = Image::new(
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
        PixelType::U8x4,
    );
    let mut dst_image = Image::new(
        NonZeroU32::new(new_width).unwrap(),
        NonZeroU32::new(new_height).unwrap(),
        PixelType::U8x4,
    );
    let mut dst_view = dst_image.view_mut();
    let mul_div = MulDiv::default();

    let mut fast_resizer = Resizer::new(ResizeAlg::Nearest);

    mul_div.multiply_alpha(&src_view, &mut premultiplied_src_image.view_mut())?;
    fast_resizer.resize(&premultiplied_src_image.view(), &mut dst_view)?;
    mul_div.divide_alpha_inplace(&mut dst_view)?;

    Ok(ImageBuffer::from_raw(new_width, new_height, dst_image.buffer().to_vec()).unwrap())
}

/// Get the actual size from the current size and the max size
/// If either width nor height is smaller or equal to max_width and max_height, the new size is
/// reduced to the larger of the two. If one of the max values is set to 0, the size in that dimension
/// is not restricted
pub fn restrict_size(
    (width, height): (u32, u32),
    (max_width, max_height): (u32, u32),
) -> (u32, u32) {
    if (width > max_width || height > max_height) && (max_width != 0 || max_height != 0) {
        let wratio = if max_width > 0 {
            max_width as f32 / width as f32
        } else {
            f32::MAX
        };
        let hratio = if max_height > 0 {
            max_height as f32 / height as f32
        } else {
            f32::MAX
        };

        let ratio = f32::min(wratio, hratio);

        let new_width = max((width as f32 * ratio).round() as u32, 1);
        let new_height = max((height as f32 * ratio).round() as u32, 1);
        (new_width, new_height)
    } else {
        (width, height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize() {
        let image_buffer = ImageBuffer::new(100, 100);
        let result = resize_image(image_buffer, 200, 100);
        assert!(result.is_ok());
        let resized_image = result.unwrap();
        assert_eq!(resized_image.width(), 200);
        assert_eq!(resized_image.height(), 100);

        let image_buffer = ImageBuffer::new(100, 100);
        let result = resize_image(image_buffer, 100, 200);
        assert!(result.is_ok());
        let resized_image = result.unwrap();
        assert_eq!(resized_image.width(), 100);
        assert_eq!(resized_image.height(), 200);
    }

    #[test]
    fn test_get_size() {
        let size = restrict_size((100, 100), (100, 100));
        assert_eq!(size, (100, 100));

        let size = restrict_size((1000, 1000), (100, 100));
        assert_eq!(size, (100, 100));

        let size = restrict_size((10, 10), (100, 100));
        assert_eq!(size, (10, 10));

        let size = restrict_size((100, 50), (100, 100));
        assert_eq!(size, (100, 50));

        let size = restrict_size((50, 100), (100, 100));
        assert_eq!(size, (50, 100));

        let size = restrict_size((200, 60), (100, 100));
        assert_eq!(size, (100, 30));

        let size = restrict_size((200, 60), (100, 0));
        assert_eq!(size, (100, 30));

        let size = restrict_size((60, 150), (100, 100));
        assert_eq!(size, (40, 100));

        let size = restrict_size((60, 150), (0, 100));
        assert_eq!(size, (40, 100));

        let size = restrict_size((200, 400), (100, 100));
        assert_eq!(size, (50, 100));

        let size = restrict_size((400, 200), (100, 100));
        assert_eq!(size, (100, 50));

        let size = restrict_size((400, 200), (0, 0));
        assert_eq!(size, (400, 200));
    }
}
