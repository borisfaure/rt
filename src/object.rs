use crate::raytracer::Ray;

pub trait Object {
    fn intersects(&self, r: Ray) -> bool;
}
