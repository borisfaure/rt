use image::{
    Rgb,
};
use crate::raytracer::{
    Ray,
    Shading
};
use crate::maths::Coords;

pub trait Object {
    fn intersects(&self, r: &Ray) -> Option<(f64, Shading)>;
}

pub struct Sphere {
    pub center: Coords,
    pub radius: f64,
    pub color: Rgb<u8>,
}

impl Object for Sphere {
    fn intersects(&self, _r: &Ray) -> Option<(f64, Shading)> {
        return None;
    }
}
