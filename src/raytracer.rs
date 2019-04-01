use image::{
    RgbImage,
};
use crate::scene::Scene;

pub struct Ray {
}

pub fn render_scene(scene: Scene, img: &RgbImage) {
    // Obtain the image's width and height.
    let (width, height) = img.dimensions();
}
