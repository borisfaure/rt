extern crate image;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate color_scaling;
extern crate chrono;
#[macro_use]
extern crate clap;
extern crate rand;
extern crate rayon;
extern crate regex;
#[macro_use]
extern crate erased_serde;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use clap::{
   App,
   Arg,
   SubCommand,
};
use image::{
    Rgb,
    RgbImage
};
use regex::Regex;

mod maths;
mod object;
mod scene;
mod raytracer;


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
    Plan,
};


struct Preset {
    eye: Eye,
    nb_samples: u64,
    screen: Screen,
    density: f64,
}

fn parse_geometry(val: &str) -> Result<(u32,u32), String> {
    let re = Regex::new(r"(\d+)x(\d+)").unwrap();
    if let Some(c) = re.captures(val) {
        let x : u32 = c.get(1).unwrap().as_str().parse::<u32>().unwrap();
        let y : u32 = c.get(2).unwrap().as_str().parse::<u32>().unwrap();
        Ok((x,y))
    } else {
        Err("invalid geometry".to_owned())
    }
}
fn is_geometry(val: String) -> Result<(), String> {
    match parse_geometry(&val) {
        Err(s) => Err(s),
        _ => Ok(())
    }
}

fn parse_vec3(val: &str) -> Result<Vec3, String> {
    let re = Regex::new(r"\(([+-]?[0-9]+[.][0-9]*),[ ]+([+-]?[0-9]+[.][0-9]*),[ ]+([+-]?[0-9]+[.][0-9]*)\)").unwrap();
    if let Some(c) = re.captures(&val) {
        let a : f64 = c.get(1).unwrap().as_str().parse::<f64>().unwrap();
        let b : f64 = c.get(2).unwrap().as_str().parse::<f64>().unwrap();
        let c : f64 = c.get(3).unwrap().as_str().parse::<f64>().unwrap();
        Ok(Vec3::new(a, b, c))
    } else {
        Err("invalid vector".to_owned())
    }
}
fn is_vec3(val: String) -> Result<(), String> {
    match parse_vec3(&val) {
        Err(s) => Err(s),
        _ => Ok(())
    }
}

fn main() {
    pretty_env_logger::init();
    let m = App::new("Ray Tracer")
        .version("0.1.0")
        .author("Boris Faure <billiob@gmail.com>")
        .about("Generate ray traced images")
        .subcommand(SubCommand::with_name("forest")
                    .about("construct a json file of a forest scene")
                    .arg(Arg::with_name("CFG")
                         .help("config file")
                         .required(true)
                         .index(1))
                    .arg(Arg::with_name("density")
                         .short("d")
                         .long("density")
                         .takes_value(true)
                         .default_value("0.1")
                         .help("forest density"))
                    .arg(Arg::with_name("golden_sun")
                         .long("golden-sun")
                         .help("add a golden sun"))
                    .arg(Arg::with_name("blue_sun")
                         .long("blue-sun")
                         .help("add a blue sun"))
                    .arg(Arg::with_name("geometry")
                         .short("g")
                         .long("geometry")
                         .default_value("512x270")
                         .validator(is_geometry)
                         .help("size of the image that would be generated"))
                    .arg(Arg::with_name("eye_position")
                         .short("e")
                         .long("eye-position")
                         .default_value("(0., 25., 0.)")
                         .validator(is_vec3)
                         .help("position of the eye in the scene"))
                    .arg(Arg::with_name("eye_direction")
                         .short("i")
                         .long("eye-direction")
                         .default_value("(0., -1.0, 1.)")
                         .validator(is_vec3)
                         .help("direction of the eye in the scene"))
                    .arg(Arg::with_name("floor")
                         .short("f")
                         .long("floor")
                         .default_value("(0.0, 1.0, -1.0)")
                         .validator(is_vec3)
                         .help("normal direction of the floor in the scene"))
                    )
        .subcommand(SubCommand::with_name("render")
                    .about("renders a scene")
                    .arg(Arg::with_name("CFG")
                         .help("config file")
                         .required(true)
                         .index(1))
                    .arg(Arg::with_name("PNG")
                         .help("png file to render to")
                         .required(true)
                         .index(2))
                    .arg(Arg::with_name("samples")
                         .short("s")
                         .long("samples")
                         .takes_value(true)
                         .default_value("8")
                         .help("number of rays to fire per pixel"))
                    .arg(Arg::with_name("geometry")
                         .short("g")
                         .long("geometry")
                         .default_value("4096x2160")
                         .validator(is_geometry)
                         .help("size of the image that would be generated"))
                    )
        .get_matches();

    if let Some(m) = m.subcommand_matches("test") {
        let cfgpath = m.value_of("CFG").unwrap();
        let density = value_t!(m, "density", f64).unwrap();
        let (w,h) = parse_geometry(m.value_of("geometry").unwrap()).unwrap();
        let eye_pos = parse_vec3(m.value_of("eye_position").unwrap()).unwrap();
        let eye_dir = parse_vec3(m.value_of("eye_direction").unwrap()).unwrap();
        let floor_dir = parse_vec3(m.value_of("floor").unwrap()).unwrap();
        let mut scene = Scene::new();

        if m.is_present("golden_sun") {
            /* golden hour */
            scene.set_golden_sun();
        }
        if m.is_present("blue_sun") {
            scene.set_blue_sun();
        }
        let preset = Preset {
            eye: Eye {
                origin: eye_pos,
                direction: eye_dir
            },
            nb_samples: 8_u64,
            screen: Screen { width: w, height: h},
            density: density,
        };

        let ray_ctx = RayCtx::new(&preset.eye, &preset.screen);

        let floor = Plan::new(
            Vec3::origin(),
            floor_dir,
            Rgb([237, 201, 175])
            );
        let footprint = ray_ctx.get_footprint(&floor);
        info!("footprint:{:?}", footprint);
        scene.add(floor);
        let trees = scene.generate_forest_monte_carlo(&footprint,
                                                      preset.density);
        info!("trees:{:?}", trees);
        scene.add_signature(&ray_ctx);

        scene.save(cfgpath);
    } else if let Some(m) = m.subcommand_matches("render") {
        let pngpath = m.value_of("PNG").unwrap();
        let cfgpath = m.value_of("CFG").unwrap();
        let samples = 128_u64;

        let small = false;
        let preset;
        if small {
            preset = Preset {
                eye: Eye { origin: Vec3::new(0., 25., 0.),
                direction: Vec3::new_normalized(0., -1.0, 1.)
                },
                nb_samples: samples,
                screen: Screen { width: 512, height: 256 },
                density: 0.1,
            }
        } else {
            preset = Preset {
                eye: Eye { origin: Vec3::new(0., 25., 0.),
                direction: Vec3::new_normalized(0., -1.0, 1.)
                },
                nb_samples: 128_u64,
                screen: Screen { width: 4096, height: 2160},
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
            Rgb([237, 201, 175])
            );
        let footprint = ray_ctx.get_footprint(&floor);
        info!("footprint:{:?}", footprint);
        scene.add(floor);

        let img : RgbImage = ray_ctx.render_scene(&scene, preset.nb_samples);

        img.save(pngpath).unwrap();
    }
}
