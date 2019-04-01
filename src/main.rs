extern crate image;
use image::{
    ImageBuffer,
    RgbImage
};

mod object;
mod scene;
mod raytracer;

use std::path;
use scene::{
    Coord,
    Scene
};

fn main() {
    let path = path::Path::new("/tmp/test_raytracer.png");

    // Construct a new ImageBuffer with the specified width and height.
    let img : RgbImage = ImageBuffer::new(512, 512);

    let camera = Coord {x: 1., y: 1., z: 1.};
    let scene = Scene::new(camera);

    raytracer::render_scene(scene, &img);

    img.save(path).unwrap();
}
