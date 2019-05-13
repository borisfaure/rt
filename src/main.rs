extern crate image;
#[macro_use]
extern crate log;
extern crate chrono;
extern crate color_scaling;
extern crate pretty_env_logger;
extern crate rand;
extern crate rayon;

use image::{Rgb, RgbImage};

mod maths;
mod object;
mod raytracer;
mod scene;

use maths::Vec3;
use object::{Ellipsoid, Owl, Plan, Signature, Sphere};
use raytracer::{Eye, RayCtx, Screen};
use scene::Scene;
use std::path;
struct Preset {
    eye: Eye,
    nb_samples: u64,
    screen: Screen,
    density: f64,
}

fn main() {
    pretty_env_logger::init();
    let path = path::Path::new("/tmp/test_raytracer.png");

    let small = false;
    let preset;
    if small {
        preset = Preset {
            eye: Eye {
                origin: Vec3::new(0., 25., 0.),
                direction: Vec3::new_normalized(0., -1.0, 1.),
            },
            nb_samples: 8_u64,
            screen: Screen {
                width: 512,
                height: 256,
            },
            density: 0.1,
        }
    } else {
        preset = Preset {
            eye: Eye {
                origin: Vec3::new(0., 25., 0.),
                direction: Vec3::new_normalized(0., -1.0, 1.),
            },
            nb_samples: 128_u64,
            screen: Screen {
                width: 4096,
                height: 2160,
            },
            density: 0.4,
        }
    }

    let mut scene = Scene::new();

    /* golden hour */
    scene.set_golden_sun();

    let ray_ctx = RayCtx::new(&preset.eye, &preset.screen);

    let floor = Plan::new(
        Vec3::origin(),
        Vec3::new(0.0, 1.0, -1.0),
        Rgb([237, 201, 175]),
    );
    let footprint = ray_ctx.get_footprint(&floor);
    info!("footprint:{:?}", footprint);
    scene.add(floor);
    let trees = scene.generate_forest_monte_carlo(&footprint, preset.density, false);
    info!("trees:{:?}", trees);
    let signature = Signature::new(&ray_ctx);
    scene.add(signature);

    let img: RgbImage = ray_ctx.render_scene(&scene, preset.nb_samples);

    img.save(path).unwrap();
}
