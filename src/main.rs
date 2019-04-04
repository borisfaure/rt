extern crate image;

use image::{
    ImageBuffer,
    Rgb,
    RgbImage
};

mod maths;
mod object;
mod scene;
mod raytracer;

use std::path;
use raytracer::{
    Eye,
};
use scene::{
    Scene
};
use maths::{
    Coords,
    Vector,
};

fn main() {
    let path = path::Path::new("/tmp/test_raytracer.png");

    // Construct a new ImageBuffer with the specified width and height.
    let mut img : RgbImage = ImageBuffer::new(512, 512);

    let eye = Eye { origin: Coords::new(2., 2., 2.),
                    direction: Vector::new(1., 1., 1.),};

    let mut scene = Scene::new();
    let sphere = object::Sphere {
        center: Coords {x: 0.4, y: 0.5, z: 0.6},
        radius: 0.3,
        color: Rgb([255, 0, 216]),
    };
    scene.add(sphere);


    raytracer::render_scene(&scene, &eye, &mut img);

    img.save(path).unwrap();
}
