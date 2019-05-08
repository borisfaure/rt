extern crate image;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate color_scaling;
extern crate chrono;
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
    RayCtx,
    Screen,
};
use scene::{
    Scene
};
use maths::{
    Vec3,
};
use object::{
    Ellipsoid,
    Owl,
    Plan,
    Sphere,
};

fn main() {
    pretty_env_logger::init();
    let path = path::Path::new("/tmp/test_raytracer.png");

    let eye = Eye { origin: Vec3::new(0., 25., 0.),
                    direction: Vec3::new_normalized(0., -1.0, 1.)
    };

    let mut scene = Scene::new();

    /* golden hour */
    scene.set_golden_sun();


    let nb_samples = 256_u64;
    let screen = Screen { width: 512, height: 256 };

    let ray_ctx = RayCtx::new(&eye, &screen);
    {
        let sphere = Sphere::new(
            Vec3::new(-5., 15.5, 13.),
            0.5,
            Rgb([255, 0, 0])
            );
        scene.add(sphere);
    }
    {
        let sphere = Sphere::new(
            Vec3::new(-5., 14.5, 13.),
            0.5,
            Rgb([255, 0, 216])
            );
        scene.add(sphere);
    }
    {
        let sphere = Sphere::new(
            Vec3::new(-5., 13.5, 13.),
            0.5,
            Rgb([255, 127, 0])
            );
        scene.add(sphere);
    }

    {
        let owl = Owl::new(
            Vec3::new(0., 12., 10.),
            6.,
            );
        scene.add(owl);
    }

    {

        let sphere = Sphere::new(
            Vec3::new(5., 12.5, 10.),
            0.5,
            Rgb([255, 0, 0])
            );
        scene.add(sphere);
    }
    {
        let sphere = Sphere::new(
            Vec3::new(5., 11.5, 10.),
            0.5,
            Rgb([255, 0, 216])
            );
        scene.add(sphere);
    }
    {
        let sphere = Sphere::new(
            Vec3::new(5., 10.5, 10.),
            0.5,
            Rgb([255, 127, 0])
            );
        scene.add(sphere);
    }


    let floor = Plan::new(
        Vec3::origin(),
        Vec3::new(0.0, 1.0, -1.0),
        Rgb([34, 139, 34])
        //Rgb([237, 201, 175])
        );
    let footprint = ray_ctx.get_footprint(&floor);
    info!("footprint:{:?}", footprint);
    scene.add(floor);
    //let trees = scene.generate_forest_monte_carlo(&footprint, 0.30);
    //info!("trees:{:?}", trees);

    let img : RgbImage = ray_ctx.render_scene(&scene, nb_samples);

    img.save(path).unwrap();
}
