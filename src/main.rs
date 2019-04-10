extern crate image;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate color_scaling;
extern crate rand;

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
    Vec3,
};
use object::{
    Plan,
    Sphere,
};

fn main() {
    pretty_env_logger::init();
    let path = path::Path::new("/tmp/test_raytracer.png");

    // Construct a new ImageBuffer with the specified width and height.
    let mut img : RgbImage = ImageBuffer::new(512, 256);
    //let mut img : RgbImage = ImageBuffer::new(10, 10);

    let eye = Eye { origin: Vec3::new(0., 1., -3.),
                    direction: Vec3::new_normalized(0., -0.1, 1.)
    };

    let mut scene = Scene::new();
    let sphere = Sphere::new(
        Vec3::new(0.0, 0.4, 0.0),
        0.4,
        Rgb([255, 0, 216])
    );
    scene.add(sphere);

    let floor = Plan::new(
        Vec3::origin(),
        Vec3::new(0.0, 1.0, 0.0),
        Rgb([237, 201, 175])
    );
    scene.add(floor);

    raytracer::render_scene(&scene, &eye, 128_u64, &mut img);

    img.save(path).unwrap();
}
