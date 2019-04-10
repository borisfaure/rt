use color_scaling::scale_rgb;
use image::{
    Rgb,
};
use crate::raytracer::{
    Ray,
    Hit,
};
use crate::maths::{
    remap_01,
    Vec3,
};

pub trait Object {
    fn hits(&self, r: &Ray) -> Option<Hit>;
}

pub struct Plan {
    p: Vec3,
    normal: Vec3,
    color: Rgb<u8>,
}
impl Plan {
    pub fn new(p: Vec3, normal: Vec3, color: Rgb<u8>) -> Plan {
        Plan {
            p: p,
            normal: normal,
            color: color,
        }
    }
}
impl Object for Plan {
    fn hits(&self, r: &Ray) -> Option<Hit> {
        None
    }
}


pub struct Sphere {
    center: Vec3,
    radius: f64,
    rd_sq: f64,
    color: Rgb<u8>,
}
impl Sphere {
    pub fn new(center: Vec3, radius: f64, color: Rgb<u8>) -> Sphere {
        Sphere {
            center: center,
            radius: radius,
            rd_sq: radius * radius,
            color: color
        }
    }
}

impl Object for Sphere {
    fn hits(&self, r: &Ray) -> Option<Hit> {
        let v = Vec3::new(self.center.x - r.o.x,
                            self.center.y - r.o.y,
                            self.center.z - r.o.z);
        let t = v.dot_product(&r.d);
        let p = Vec3::new(r.o.x + t * r.d.x,
                          r.o.y + t * r.d.y,
                          r.o.z + t * r.d.z);
        let y_sq = self.center.length_sq(&p);
        debug!("t:{:?} p:{:?}", t, p);
        debug!("y_sq:{:?} vs rd_sq:{:?}", y_sq, self.rd_sq);
        if y_sq > self.rd_sq {
            return None;
        }
        let x = f64::sqrt(self.rd_sq - y_sq);
        let t1 = t - x;
        //let t2 = t + x;
        let to_center = f64::sqrt(v.dot_product(&v));
        let s = remap_01(to_center, to_center - self.radius, t1);
        let p = Vec3::new(r.o.x + t1 * r.d.x,
                          r.o.y + t1 * r.d.y,
                          r.o.z + t1 * r.d.z);
        let black : Rgb<u8> = Rgb([0, 0, 0]);
        let h = Hit {
            color: scale_rgb(&black, &self.color, s).unwrap(),
            p: p,
            normal: Vec3::new(0., 0., 0.),
            t: t1,
        };
        Some(h)
    }
}
