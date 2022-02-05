use std::num::NonZeroU32;

use fast_image_resize::{
    DifferentTypesOfPixelsError, Image, ImageBufferError, MulDiv, MulDivImageError,
    MulDivImagesError, PixelType, ResizeAlg, Resizer,
};

use super::images::ImageBuffer;

/// We do not really care about the underlying error, so wrap all fast_image_resize errors to a single type
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
