extern crate image;
use image::{
    ImageBuffer,
    Rgb,
    RgbImage
};

mod object;
mod scene;
mod raytracer;

use std::path;
use scene::{
    Coords,
    Scene
};

fn main() {
    let path = path::Path::new("/tmp/test_raytracer.png");

    // Construct a new ImageBuffer with the specified width and height.
    let mut img : RgbImage = ImageBuffer::new(512, 512);

    let eye = Coords {x: 2., y: 2., z: 2.};
    let mut scene = Scene::new(eye);
    let sphere = object::Sphere {
        center: Coords {x: 0.4, y: 0.5, z: 0.6},
        radius: 0.3,
        color: Rgb([255, 0, 216]),
    };
    scene.add(sphere);


    raytracer::render_scene(&scene, &mut img);

    img.save(path).unwrap();
}
