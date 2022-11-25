use std::num::NonZeroU32;

use fast_image_resize as fir;
use fast_image_resize::{Image, MulDiv, PixelType};
use fir::ResizeAlg;
use image::{ColorType, DynamicImage};

/// Scales an image to be its biggest side that fits within the given dimensions.
pub fn scale_image(
    img: &DynamicImage,
    width: u32,
    height: u32,
    resize_alg: ResizeAlg,
) -> Image<'static> {
    let mut image = prepare_image_resize(img);
    let aspect = image.width().get() as f32 / image.height().get() as f32;

    if img.color().has_alpha() {
        MulDiv::default()
            .multiply_alpha_inplace(&mut image.view_mut())
            .unwrap();
    }

    let (dst_width, dst_height) = size_image(aspect, width, height);
    let mut dst_image = Image::new(
        NonZeroU32::new(dst_width).unwrap(),
        NonZeroU32::new(dst_height).unwrap(),
        image.pixel_type(),
    );
    let mut dst_view = dst_image.view_mut();

    let mut resizer = fir::Resizer::new(resize_alg);
    resizer.resize(&image.view(), &mut dst_view).unwrap();

    if img.color().has_alpha() {
        MulDiv::default()
            .divide_alpha_inplace(&mut dst_view)
            .unwrap();
    }

    dst_image
}

fn size_image(aspect: f32, width: u32, height: u32) -> (u32, u32) {
    let (w1, h1) = (width, (width as f32 / aspect) as u32);
    let (w2, h2) = (((height as f32) * aspect) as u32, height);

    if w1 <= width && h1 <= height {
        return (w1, h1);
    }
    (w2, h2)
}

fn prepare_image_resize(image: &DynamicImage) -> Image<'static> {
    let width = NonZeroU32::new(image.width()).unwrap();
    let height = NonZeroU32::new(image.height()).unwrap();

    let (bytes, color_type) = match image.color() {
        ColorType::L8 => (image.to_luma8().into_raw(), PixelType::U8),
        ColorType::L16 => (
            image
                .to_luma16()
                .into_raw()
                .iter()
                .flat_map(|&x| x.to_le_bytes())
                .collect(),
            PixelType::U16,
        ),
        ColorType::La8 => (image.to_luma_alpha8().into_raw(), PixelType::U8x2),
        ColorType::La16 => (
            image
                .to_luma_alpha16()
                .into_raw()
                .iter()
                .flat_map(|&x| x.to_le_bytes())
                .collect(),
            PixelType::U16x2,
        ),
        ColorType::Rgb8 => (image.to_rgb8().into_raw(), PixelType::U8x3),
        ColorType::Rgb16 => (
            image
                .to_rgb16()
                .into_raw()
                .iter()
                .flat_map(|&x| x.to_le_bytes())
                .collect(),
            PixelType::U16x3,
        ),
        ColorType::Rgba8 => (image.to_rgba8().into_raw(), PixelType::U8x4),
        ColorType::Rgba16 => (
            image
                .to_rgba16()
                .into_raw()
                .iter()
                .flat_map(|&x| x.to_le_bytes())
                .collect(),
            PixelType::U16x4,
        ),
        _ => panic!("Unsupported image format"),
    };

    let resize_image = Image::from_vec_u8(width, height, bytes, color_type).unwrap();
    // TODO: colorspace conversion?

    resize_image
}
