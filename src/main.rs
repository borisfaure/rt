extern crate image;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate color_scaling;
extern crate rand;
extern crate rayon;

use image::{
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
    Triangle,
    Tetrahedron,
};
use std::f64::consts::PI;

fn main() {
    pretty_env_logger::init();
    let path = path::Path::new("/tmp/test_raytracer.png");

    let eye = Eye { origin: Vec3::new(0., 1., -3.),
                    direction: Vec3::new_normalized(0., -0.1, 1.)
    };

    let mut scene = Scene::new();

    {
        let sphere = Sphere::new(
            Vec3::new(-3., 2.5, 4.),
            0.5,
            Rgb([255, 0, 0])
            );
        scene.add(sphere);
    }
    {
        let sphere = Sphere::new(
            Vec3::new(-3., 1.5, 4.),
            0.5,
            Rgb([255, 0, 216])
            );
        scene.add(sphere);
    }
    {
        let sphere = Sphere::new(
            Vec3::new(-3., 0.5, 4.),
            0.5,
            Rgb([255, 127, 0])
            );
        scene.add(sphere);
    }

    /*
    {
        let sphere = Sphere::new(
            Vec3::new(0., 2.5, 4.),
            0.5,
            Rgb([255, 0, 0])
            );
        scene.add(sphere);
    }
    {
        let sphere = Sphere::new(
            Vec3::new(0., 1.5, 4.),
            0.5,
            Rgb([255, 0, 216])
            );
        scene.add(sphere);
    }
    {
        let sphere = Sphere::new(
            Vec3::new(0., 0.5, 4.),
            0.5,
            Rgb([255, 127, 0])
            );
        scene.add(sphere);
    }
    {
        let triangle = Triangle::new(
            Vec3::new(-2., 2., 4.),
            Vec3::new(-2., -1., 4.),
            Vec3::new(2., -1., 4.),
            Rgb([255, 0, 0])
        );
        scene.add(triangle);
    }
    */
    {
        let tetrahedron = Tetrahedron::new(
            Vec3::new(0., 3., 4.),
            3., 2., 5. * PI/6.,
            Rgb([34, 139, 34])
            );
        scene.add(tetrahedron);
    }
    /*
    {
        let triangle = Triangle::new(
            Vec3::new(-0.00000000000000036739403974420594, 0., 2.),
            Vec3::new(1.732050807568878, 0., 4.999999999999998),
            Vec3::new(0., 3., 4.),
            Rgb([255, 0, 0])
        );
        scene.add(triangle);
    }
    */
    {
        let sphere = Sphere::new(
            Vec3::new(3., 2.5, 4.),
            0.5,
            Rgb([255, 0, 0])
            );
        scene.add(sphere);
    }
    {
        let sphere = Sphere::new(
            Vec3::new(3., 1.5, 4.),
            0.5,
            Rgb([255, 0, 216])
            );
        scene.add(sphere);
    }
    {
        let sphere = Sphere::new(
            Vec3::new(3., 0.5, 4.),
            0.5,
            Rgb([255, 127, 0])
            );
        scene.add(sphere);
    }
    {
        let floor = Plan::new(
            Vec3::origin(),
            Vec3::new(0.0, 1.0, 0.0),
            Rgb([237, 201, 175])
            );
        scene.add(floor);

    }

    let img : RgbImage = raytracer::render_scene(&scene,
                                                 &eye,
                                                 128_u64,
                                                 512, 256);

    img.save(path).unwrap();
}
