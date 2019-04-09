use image::{
    Rgb,
};
use crate::raytracer::{
    Ray,
    Shading
};
use crate::maths::{
    Coords,
    Vector,
};

pub trait Object {
    fn intersects(&self, r: &Ray) -> Option<(f64, Shading)>;
}

pub struct Sphere {
    center: Coords,
    radius: f64,
    rd_sq: f64,
    color: Rgb<u8>,
}
impl Sphere {
    pub fn new(center: Coords, radius: f64, color: Rgb<u8>) -> Sphere {
        Sphere {
            center: center,
            radius: radius,
            rd_sq: radius * radius,
            color: color
        }
    }
}

impl Object for Sphere {
    fn intersects(&self, r: &Ray) -> Option<(f64, Shading)> {
        let v = Vector::new(self.center.x - r.o.x,
                            self.center.y - r.o.y,
                            self.center.z - r.o.z);
        let t = v.dot_product(&r.d);
        let p = Coords::new(r.o.x + t * r.d.x,
                            r.o.y + t * r.d.y,
                            r.o.z + t * r.d.z);
        let y_sq = self.center.length_sq(&p);
        debug!("t:{:?} p:{:?}", t, p);
        debug!("y_sq:{:?} vs rd_sq:{:?}", y_sq, self.rd_sq);
        if y_sq > self.rd_sq {
            return None;
        }
        let sd = Shading {
            color: self.color.clone(),
        };
        Some((0., sd))
        //let x = f64::sqrt(self.rd_sq - y_sq);
        //let t1 = t - x;
        //let t2 = t + x;
    }
}
