extern crate image;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;

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
use object::{
    Sphere,
};

fn main() {
    pretty_env_logger::init();
    let path = path::Path::new("/tmp/test_raytracer.png");

    // Construct a new ImageBuffer with the specified width and height.
    let mut img : RgbImage = ImageBuffer::new(512, 512);
    //let mut img : RgbImage = ImageBuffer::new(10, 10);

    let eye = Eye { origin: Coords::new(0., 0., -3.),
                    direction: Vector::new_normalized(0., 0., 1.)
    };

    let mut scene = Scene::new();
    let sphere = Sphere::new(
        Coords::new(0.0, 0.0, 0.0),
        0.4,
        Rgb([255, 0, 216])
    );
    scene.add(sphere);


    raytracer::render_scene(&scene, &eye, &mut img);

    img.save(path).unwrap();
}
