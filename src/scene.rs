use crate::object::{
    Conifer,
    Object,
};
use crate::maths::{
    Vec3,
};
use crate::raytracer::Footprint;
use image::{
    Rgb,
};
use rand::Rng;
use std::f64::{
    self,
    consts::PI,
};


pub struct Scene {
    pub objects: Vec<Box<Object + Sync + Send>>,
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
    fn intersects_with_vect(&self, vec: &Vec<Circle>) -> bool{
        for c in vec {
            if self.intersects(c) {
                return true
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
    pub fn add<O: 'static + Object + Sync + Send>(&mut self, obj: O) {
        self.objects.push(Box::new(obj));
    }
    pub fn set_sun(&mut self, sun: Option<(Vec3, Vec3, f64)>) {
        self.sun = sun;
    }
    pub fn set_golden_sun(&mut self) {
        self.set_sun(
            Some(
                (Vec3::new(3., 1., -1.),
                 Rgb([242, 144, 45]).into(),
                0.6)));
    }
    pub fn set_blue_sun(&mut self) {
        self.set_sun(
            Some(
                (Vec3::new(-3., 1., 0.),
                 Rgb([21, 116, 196]).into(),
                0.9)));
    }

    pub fn generate_forest_monte_carlo(&mut self, footprint: &Footprint, threshold: f64) -> u32 {
        let mut rng = rand::thread_rng();
        let mut width = 1.5_f64;
        let mut r = width / 4_f64;
        let surface_max = footprint.get_surface() * threshold;
        let mut surface = 0_f64;
        let mut vec : Vec<Circle> = Vec::new();
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
                let conifer = Conifer::new(
                    pos,
                    this_width, 5_u8
                    );
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
