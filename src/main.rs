extern crate image;
#[macro_use]
extern crate log;
extern crate chrono;
extern crate color_scaling;
extern crate pretty_env_logger;
#[macro_use]
extern crate clap;
extern crate rand;
extern crate rayon;
extern crate regex;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate signal_hook;
#[macro_use]
extern crate debug_macros;

use clap::{App, Arg, SubCommand};
use image::Rgb;
use regex::Regex;

mod maths;
mod object;
mod raytracer;
mod scene;

use maths::Vec3;
use object::{BaseObject, Plan};
use raytracer::{Eye, RayCtx, Screen};
use scene::Scene;
use std::path::Path;

struct Preset {
    eye: Eye,
    nb_samples: u64,
    screen: Screen,
}

fn parse_geometry(val: &str) -> Result<(u32, u32), String> {
    let re = Regex::new(r"(\d+)x(\d+)").unwrap();
    if let Some(c) = re.captures(val) {
        let x: u32 = c.get(1).unwrap().as_str().parse::<u32>().unwrap();
        let y: u32 = c.get(2).unwrap().as_str().parse::<u32>().unwrap();
        Ok((x, y))
    } else {
        Err("invalid geometry".to_owned())
    }
}
fn is_geometry(val: String) -> Result<(), String> {
    match parse_geometry(&val) {
        Err(s) => Err(s),
        _ => Ok(()),
    }
}

fn parse_vec3(val: &str) -> Result<Vec3, String> {
    let re = Regex::new(
        r"\(([+-]?[0-9]+[.][0-9]*),[ ]+([+-]?[0-9]+[.][0-9]*),[ ]+([+-]?[0-9]+[.][0-9]*)\)",
    )
    .unwrap();
    if let Some(c) = re.captures(&val) {
        let a: f64 = c.get(1).unwrap().as_str().parse::<f64>().unwrap();
        let b: f64 = c.get(2).unwrap().as_str().parse::<f64>().unwrap();
        let c: f64 = c.get(3).unwrap().as_str().parse::<f64>().unwrap();
        Ok(Vec3::new(a, b, c))
    } else {
        Err("invalid vector".to_owned())
    }
}
fn is_vec3(val: String) -> Result<(), String> {
    match parse_vec3(&val) {
        Err(s) => Err(s),
        _ => Ok(()),
    }
}

fn parse_sun(val: &str) -> Result<Option<(Vec3,Vec3,f64)>, String> {
    let re = Regex::new(
        concat!(
        r"\(([+-]?[0-9]+[.]?[0-9]*),",
        r"[ ]+([+-]?[0-9]+[.]?[0-9]*),",
        r"[ ]+([+-]?[0-9]+[.]?[0-9]*),",
        r"[ ]+([0-9]+),",
        r"[ ]+([0-9]+),",
        r"[ ]+([0-9]+),",
        r"[ ]+([0-1][.]?[0-9]*)\)",)
    )
    .unwrap();
    if let Some(m) = re.captures(&val) {
        let a: f64 = m.get(1).unwrap().as_str().parse::<f64>().unwrap();
        let b: f64 = m.get(2).unwrap().as_str().parse::<f64>().unwrap();
        let c: f64 = m.get(3).unwrap().as_str().parse::<f64>().unwrap();
        let cr: u8 = m.get(4).unwrap().as_str().parse::<u8>().unwrap();
        let cg: u8 = m.get(5).unwrap().as_str().parse::<u8>().unwrap();
        let cb: u8 = m.get(6).unwrap().as_str().parse::<u8>().unwrap();
        let f: f64 = m.get(7).unwrap().as_str().parse::<f64>().unwrap();
        if f > 0. {
            Ok(Some((Vec3::new(a, b, c),
                    Rgb([cr, cg, cb]).into(),
                    f)))
        } else {
            Ok(None)
        }
    } else {
        Err("invalid sun".to_owned())
    }
}
fn is_sun(val: String) -> Result<(), String> {
    match parse_sun(&val) {
        Err(s) => Err(s),
        _ => Ok(()),
    }
}

fn main() {
    pretty_env_logger::init();
    let m = App::new("Ray Tracer")
        .version("0.1.0")
        .author("Boris Faure <billiob@gmail.com>")
        .about("Generate ray traced images")
        .subcommand(
            SubCommand::with_name("forest")
                .about("construct a json file of a forest scene")
                .arg(
                    Arg::with_name("CFG")
                        .help("config file")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("density")
                        .short("d")
                        .long("density")
                        .takes_value(true)
                        .default_value("0.1")
                        .help("forest density"),
                )
                .arg(
                    Arg::with_name("golden_sun")
                        .long("golden-sun")
                        .help("add a golden sun"),
                )
                .arg(
                    Arg::with_name("blue_sun")
                        .long("blue-sun")
                        .help("add a blue sun"),
                )
                .arg(
                    Arg::with_name("geometry")
                        .short("g")
                        .long("geometry")
                        .default_value("512x270")
                        .validator(is_geometry)
                        .help("size of the image that would be generated"),
                )
                .arg(
                    Arg::with_name("eye_position")
                        .short("e")
                        .long("eye-position")
                        .default_value("(0., 25., 0.)")
                        .validator(is_vec3)
                        .help("position of the eye in the scene"),
                )
                .arg(
                    Arg::with_name("eye_direction")
                        .short("i")
                        .long("eye-direction")
                        .default_value("(0., -1.0, 1.)")
                        .validator(is_vec3)
                        .help("direction of the eye in the scene"),
                )
                .arg(
                    Arg::with_name("floor")
                        .short("f")
                        .long("floor")
                        .default_value("(0.0, 1.0, -1.0)")
                        .validator(is_vec3)
                        .help("normal direction of the floor in the scene"),
                ),
        )
        .subcommand(
            SubCommand::with_name("extract")
                .about("construct a json file of from a picture")
                .arg(
                    Arg::with_name("CFG")
                        .help("config file")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("PNG")
                        .help("png file to extract from")
                        .required(true)
                        .index(2),
                )
                .arg(
                    Arg::with_name("vertical_spheres")
                        .short("n")
                        .long("vertical-spheres")
                        .takes_value(true)
                        .default_value("5.")
                        .help("Heigth represents that many spheres"),
                )
                .arg(
                    Arg::with_name("eye_position")
                        .short("e")
                        .long("eye-position")
                        .default_value("(0., 0., -25.)")
                        .validator(is_vec3)
                        .help("position of the eye in the scene"),
                )
                .arg(
                    Arg::with_name("eye_direction")
                        .short("i")
                        .long("eye-direction")
                        .default_value("(0., 0., 1.)")
                        .validator(is_vec3)
                        .help("direction of the eye in the scene"),
                )
                .arg(
                    Arg::with_name("floor")
                        .short("f")
                        .long("floor")
                        .default_value("(0.0, 1.0, 0.0)")
                        .validator(is_vec3)
                        .help("normal direction of the floor in the scene"),
                )
                .arg(
                    Arg::with_name("sun")
                        .short("s")
                        .long("sun")
                        .default_value("(0.0, 1.0, 0.0, 0, 0, 0, 0.)")
                        .validator(is_sun)
                        .help("sun"),
                )
                .arg(
                    Arg::with_name("golden_sun")
                        .long("golden-sun")
                        .help("add a golden sun"),
                )
                .arg(
                    Arg::with_name("blue_sun")
                        .long("blue-sun")
                        .help("add a blue sun"),
                ),
        )
        .subcommand(
            SubCommand::with_name("render")
                .about("renders a scene")
                .arg(
                    Arg::with_name("CFG")
                        .help("config file")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("PNG")
                        .help("png file to render to")
                        .required(true)
                        .index(2),
                )
                .arg(
                    Arg::with_name("samples")
                        .short("s")
                        .long("samples")
                        .takes_value(true)
                        .default_value("8")
                        .help("number of rays to fire per pixel"),
                )
                .arg(
                    Arg::with_name("eye_position")
                        .short("e")
                        .long("eye-position")
                        .default_value("(0., 0., -25.)")
                        .validator(is_vec3)
                        .help("position of the eye in the scene"),
                )
                .arg(
                    Arg::with_name("eye_direction")
                        .short("i")
                        .long("eye-direction")
                        .default_value("(0., 0., 1.)")
                        .validator(is_vec3)
                        .help("direction of the eye in the scene"),
                )
                .arg(
                    Arg::with_name("geometry")
                        .short("g")
                        .long("geometry")
                        .default_value("4096x2160")
                        .validator(is_geometry)
                        .help("size of the image that would be generated"),
                )
                .arg(
                    Arg::with_name("no_shadows")
                        .long("no-shadows")
                        .help("Do not render shadows"),
                )
                .arg(
                    Arg::with_name("no_lambertian")
                        .long("no-lambertians")
                        .help("Do not render lambertians"),
                ),
        )
        .get_matches();

    if let Some(m) = m.subcommand_matches("forest") {
        let cfgpath = m.value_of("CFG").unwrap();
        let density = value_t!(m, "density", f64).unwrap();
        let (w, h) = parse_geometry(m.value_of("geometry").unwrap()).unwrap();
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
                direction: eye_dir,
            },
            nb_samples: 8_u64,
            screen: Screen {
                width: w,
                height: h,
            },
        };

        let ray_ctx = RayCtx::new(&preset.eye, &preset.screen, false, false);

        let floor = Plan::new(Vec3::origin(), floor_dir, Rgb([237, 201, 175]));
        let footprint = ray_ctx.get_footprint(&floor);
        info!("footprint:{:?}", footprint);
        scene.add(BaseObject::Plan(floor));
        let trees = scene.generate_forest_monte_carlo(&footprint, density);
        info!("trees:{:?}", trees);
        scene.add_signature(&ray_ctx);

        scene.save(Path::new(cfgpath));
    } else if let Some(m) = m.subcommand_matches("extract") {
        let pngpath = m.value_of("PNG").unwrap();
        let cfgpath = m.value_of("CFG").unwrap();
        let nb_vert_spheres = value_t!(m, "vertical_spheres", f64).unwrap();
        let eye_pos = parse_vec3(m.value_of("eye_position").unwrap()).unwrap();
        let eye_dir = parse_vec3(m.value_of("eye_direction").unwrap()).unwrap();
        let floor_dir = parse_vec3(m.value_of("floor").unwrap()).unwrap();
        let mut scene = Scene::new();

        let img = image::open(pngpath).unwrap();
        let buf = img.to_rgb();

        let preset = Preset {
            eye: Eye {
                origin: eye_pos,
                direction: eye_dir,
            },
            nb_samples: 8_u64,
            screen: Screen {
                width: buf.width(),
                height: buf.height(),
            },
        };

        let sun = parse_sun(m.value_of("sun").unwrap()).unwrap();
        scene.set_sun(sun);
        if m.is_present("golden_sun") {
            /* golden hour */
            scene.set_golden_sun();
        }
        if m.is_present("blue_sun") {
            scene.set_blue_sun();
        }

        let ray_ctx = RayCtx::new(&preset.eye, &preset.screen, false, false);
        dbg!("rayctx:{:?}", ray_ctx);

        let floor = Plan::new(Vec3::origin(), floor_dir, Rgb([237, 201, 175]));
        scene.add(BaseObject::Plan(floor));
        let spheres = scene.generate_from_image(&ray_ctx, buf, nb_vert_spheres);
        info!("spheres:{:?}", spheres);
        scene.add_signature(&ray_ctx);

        scene.save(Path::new(cfgpath));
    } else if let Some(m) = m.subcommand_matches("render") {
        let pngpath = m.value_of("PNG").unwrap();
        let cfgpath = m.value_of("CFG").unwrap();
        let samples = value_t!(m, "samples", u64).unwrap();
        let (w, h) = parse_geometry(m.value_of("geometry").unwrap()).unwrap();
        let eye_pos = parse_vec3(m.value_of("eye_position").unwrap()).unwrap();
        let eye_dir = parse_vec3(m.value_of("eye_direction").unwrap()).unwrap();

        let scene = Scene::load(Path::new(cfgpath));

        let preset = Preset {
            eye: Eye {
                origin: eye_pos,
                direction: eye_dir,
            },
            nb_samples: samples,
            screen: Screen {
                width: w,
                height: h,
            },
        };

        let lambertian = !m.is_present("no_lambertian");
        let shadows = !m.is_present("no_shadows");

        let ray_ctx = RayCtx::new(&preset.eye, &preset.screen,
                                  lambertian, shadows);

        ray_ctx.render_scene(&scene, preset.nb_samples, pngpath);
    }
}
