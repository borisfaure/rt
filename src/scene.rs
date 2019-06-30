use crate::maths::Vec3;
use crate::object::{Sphere, Conifer, ObjectTrait};
use crate::raytracer::{
    Footprint,
    RayCtx,
};
use image::Rgb;
use rand::Rng;
use std::f64::{self, consts::PI};
use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::io::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct Scene {
    pub objects: Vec<Box<ObjectTrait + Sync + Send>>,
    pub sun: Option<(Vec3, Vec3, f64)>,
}


struct Circle {
    center: Vec3,
    radix: f64,
}
impl Circle {
    fn new(center: Vec3, radix: f64) -> Circle {
        Circle {
            center: center,
            radix: radix,
        }
    }
    fn intersects(&self, other: &Circle) -> bool {
        let len_sq = self.center.length_sq_to(&other.center);
        let r0 = self.radix;
        let r1 = other.radix;
        (r0 - r1) * (r0 - r1) <= len_sq && len_sq <= (r0 + r1) * (r0 + r1)
    }
    fn intersects_with_vect(&self, vec: &Vec<Circle>) -> bool {
        for c in vec {
            if self.intersects(c) {
                return true;
            }
        }
        false
    }
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            objects: Vec::new(),
            sun: None,
        }
    }
    pub fn add<O: 'static + ObjectTrait + Sync + Send>(&mut self, obj: O) {
        self.objects.push(Box::new(obj));
    }
    pub fn set_sun(&mut self, sun: Option<(Vec3, Vec3, f64)>) {
        self.sun = sun;
    }
    pub fn set_golden_sun(&mut self) {
        self.set_sun(Some((
            Vec3::new(3., 1., -3.),
            Rgb([242, 144, 45]).into(),
            0.8,
        )));
    }
    pub fn set_blue_sun(&mut self) {
        self.set_sun(Some((
            Vec3::new(-3., 1., 0.),
            Rgb([21, 116, 196]).into(),
            0.9,
        )));
    }
    pub fn save(&self, json_file_path: &Path) {
        let f = match File::create(&json_file_path) {
            Err(why) => {
                let display = json_file_path.display();
                panic!("couldn't create {}: {}", display, why.description())
            },
            Ok(file) => file,
        };
        if let Err(why) = serde_json::to_writer_pretty(f, self) {
            let display = json_file_path.display();
            panic!("couldn't create {}: {}", display, why.description())
        }
    }

    pub fn add_signature(&mut self, ray_ctx: &RayCtx) {
        /* compute radius + bottom left pos */
        let diameter = 0.008 * ray_ctx.p_bottom_right.length_sq_to(&ray_ctx.p_top_right).sqrt();
        let radius = diameter / 2.;
        let c = ray_ctx.eye.origin.translate(&ray_ctx.eye.direction,
                                             1. + 2. * diameter);
        let bottom_right = Vec3::new(
            c.x + ray_ctx.b.x - ray_ctx.v.x / ray_ctx.aspect_ratio,
            c.y + ray_ctx.b.y - ray_ctx.v.y / ray_ctx.aspect_ratio,
            c.z + ray_ctx.b.z - ray_ctx.v.z / ray_ctx.aspect_ratio);
        let base = Vec3::new(
            bottom_right.x - 25. * diameter * ray_ctx.b.x
                + 2. * diameter * ray_ctx.v.x,
            bottom_right.y - 25. * diameter * ray_ctx.b.y
                + 2. * diameter * ray_ctx.v.y,
            bottom_right.z - 25. * diameter * ray_ctx.b.z
                + 2. * diameter * ray_ctx.v.z);
        let color = Rgb([254, 55, 32]);
        let mut add_point = |x: f64, y: f64| {
            let v = Vec3::new(
                    base.x + x * diameter * ray_ctx.b.x + y * diameter * ray_ctx.v.x,
                    base.y + x * diameter * ray_ctx.b.y + y * diameter * ray_ctx.v.y,
                    base.z + x * diameter * ray_ctx.b.z + y * diameter * ray_ctx.v.z);
            let sphere = Sphere::new(
                v,
                radius,
                color.clone()
                );
            self.add(sphere);
        };
        /* B */
        add_point(0., 0.);
        add_point(0., 1.);
        add_point(0., 2.);
        add_point(0., 3.);
        add_point(0., 4.);
        add_point(1., 0.);
        add_point(1., 2.);
        add_point(1., 4.);
        add_point(2., 1.);
        add_point(2., 3.);
        /* . */
        add_point(4., 0.);
        /* F */
        add_point(6., 0.);
        add_point(6., 1.);
        add_point(6., 2.);
        add_point(6., 3.);
        add_point(6., 4.);
        add_point(7., 2.);
        add_point(7., 4.);
        add_point(8., 4.);
        /* A */
        add_point(10., 0.);
        add_point(10., 1.);
        add_point(10., 2.);
        add_point(10., 3.);
        add_point(11., 2.);
        add_point(11., 4.);
        add_point(12., 0.);
        add_point(12., 1.);
        add_point(12., 2.);
        add_point(12., 3.);
        /* U */
        add_point(14., 0.);
        add_point(14., 1.);
        add_point(14., 2.);
        add_point(14., 3.);
        add_point(14., 4.);
        add_point(15., 0.);
        add_point(16., 0.);
        add_point(16., 1.);
        add_point(16., 2.);
        add_point(16., 3.);
        add_point(16., 4.);
        /* R */
        add_point(18., 0.);
        add_point(18., 1.);
        add_point(18., 2.);
        add_point(18., 3.);
        add_point(18., 4.);
        add_point(19., 2.);
        add_point(19., 4.);
        add_point(20., 0.);
        add_point(20., 1.);
        add_point(20., 3.);
        /* E */
        add_point(22., 0.);
        add_point(22., 1.);
        add_point(22., 2.);
        add_point(22., 3.);
        add_point(22., 4.);
        add_point(23., 0.);
        add_point(23., 2.);
        add_point(23., 4.);
        add_point(24., 0.);
        add_point(24., 4.);
    }

    pub fn generate_forest_monte_carlo(
        &mut self,
        footprint: &Footprint,
        threshold: f64
    ) -> u32 {
        let mut rng = rand::thread_rng();
        let mut width = 1.5_f64;
        let mut r = width / 4_f64;
        let surface_max = footprint.get_surface() * threshold;
        let mut surface = 0_f64;
        let mut vec: Vec<Circle> = Vec::new();
        let mut trees = 0_u32;
        let mut tries = 0_u32;
        let decreasing_factor = 0.8_f64;

        loop {
            let i = rng.gen::<f64>();
            let j = rng.gen::<f64>();
            let pos = footprint.get_real_position(i, j);
            let rnd_factor = 0.7_f64 + 0.6_f64 * rng.gen::<f64>();
            let this_r = r * rnd_factor;
            let this_width = width * rnd_factor;
            let c = Circle::new(pos.clone(), r);

            if c.intersects_with_vect(&vec) {
                tries += 1;
                if tries > 50 {
                    width *= decreasing_factor;
                    r *= decreasing_factor;
                }
            } else {
                tries = 0;
                let conifer = Conifer::new(pos, this_width, 5_u8);
                self.add(conifer);
                vec.push(c);
                trees += 1;
                surface += PI * this_r * this_r;
                if surface >= surface_max {
                    break;
                }
            }
        }
        trees
    }
}
