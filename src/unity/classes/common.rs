use image::{DynamicImage, RgbaImage};

use crate::unity::{object::ObjectInfo, Result, FromObject, Error, Object, Reader};



pub fn image_alpha_merge(mut main: RgbaImage, alpha: RgbaImage) -> RgbaImage {
    let alpha_rsize_dyn: DynamicImage;
    let mut alpha_resize = &alpha;
    if main.dimensions() != alpha.dimensions() {
        let alpha_dyn: DynamicImage = alpha.into();
        alpha_rsize_dyn = alpha_dyn.resize(
            main.width(),
            main.height(),
            image::imageops::FilterType::Nearest,
        );
        alpha_resize = alpha_rsize_dyn.as_rgba8().unwrap()
    }
    for (x, y, pixel) in main.enumerate_pixels_mut() {
        pixel[3] = alpha_resize.get_pixel(x, y)[0];
    }
    main
}