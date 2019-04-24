use color_scaling::scale_rgb;
use image::{
    Rgb,
};
use crate::raytracer::{
    Ray,
    Hit,
};
use crate::maths::{
    EPSILON,
    remap_01,
    Vec3,
    solve_3variable_system,
};


pub trait Object {
    fn hits(&self, r: &Ray, tmin: f64, tmax: f64) -> Option<Hit>;
}

/* {{{ Plan */
pub struct Plan {
    p: Vec3,
    normal: Vec3,
    color: Vec3,
}
impl Plan {
    pub fn new(p: Vec3, normal: Vec3, color: Rgb<u8>) -> Plan {
        Plan {
            p: p,
            normal: normal,
            color: color.into(),
        }
    }
}
impl Object for Plan {
    fn hits(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<Hit> {
        let dn = ray.direction.dot_product(&self.normal);
        if dn >= EPSILON {
            return None;
        }
        let to_plan = ray.origin.to(&self.p);
        let t = to_plan.dot_product(&self.normal) / dn;
        if t <= tmin || t >= tmax {
            return None;
        }
        let p = ray.at(t);
        let h = Hit {
            color: self.color.clone(),
            normal: self.normal.clone(),
            p: p,
            t: t,
        };
        Some(h)
    }
}

/* }}} */
/* {{{ Sphere */
pub struct Sphere {
    center: Vec3,
    radius: f64,
    rd_sq: f64,
    color: Vec3,
}
impl Sphere {
    pub fn new(center: Vec3, radius: f64, color: Rgb<u8>) -> Sphere {
        Sphere {
            center: center,
            radius: radius,
            rd_sq: radius * radius,
            color: color.into()
        }
    }
}

impl Object for Sphere {
    fn hits(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<Hit> {
        let oc = self.center.to(&ray.origin);
        let a = ray.direction.dot_product(&ray.direction);
        let b = oc.dot_product(&ray.direction);
        let c = oc.dot_product(&oc) - self.rd_sq;
        let discrimant = b*b - a*c;
        if discrimant <= 0_f64 {
            return None;
        }
        let discrimant_sqrt = discrimant.sqrt();
        let t1 = (-b - discrimant_sqrt ) / a;
        if tmin < t1 && t1 < tmax {
            let p = ray.at(t1);
            let mut n = self.center.to(&p);
            n.div(self.radius);
            let h = Hit {
                color: self.color.clone(),
                normal: n,
                p: p,
                t: t1,
            };
            return Some(h);
        }
        let t2 = (-b + discrimant_sqrt ) / a;
        if tmin < t2 && t2 < tmax {
            let p = ray.at(t2);
            let mut n = self.center.to(&p);
            n.div(self.radius);
            let h = Hit {
                color: self.color.clone(),
                normal: n,
                p: p,
                t: t2,
            };
            return Some(h);
        }
        None
    }
}

/* }}} */
/* Triangle {{{ */

pub struct Triangle {
    a: Vec3,
    b: Vec3,
    c: Vec3,
    normal: Vec3,
    color: Vec3,
}
impl Triangle {
    pub fn new(a: Vec3, b: Vec3, c: Vec3, color: Rgb<u8>) -> Triangle {
        let ab = a.to(&b);
        let ac = a.to(&c);
        let mut normal = ab.cross_product(&ac);
        normal.normalize();
        Triangle {
            a: a,
            b: b,
            c: c,
            normal: normal,
            color: color.into(),
        }
    }
}
impl Object for Triangle {
    fn hits(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<Hit> {
        /* find intersection with the plan */
        let dn = ray.direction.dot_product(&self.normal);
        if dn >= 0. {
            return None;
        }
        let to_plan = ray.origin.to(&self.a);
        let t = to_plan.dot_product(&self.normal) / dn;
        if t <= tmin || t >= tmax {
            return None;
        }
        let p = ray.at(t);
        let o = solve_3variable_system(&self.a, &self.b, &self.c, &p);
        if let Some(w) = o {
            if w.x < 0. || w.y < 0. || w.z < 0. {
                return None
            }
            error!("HIT");
            let h = Hit {
                color: self.color.clone(),
                normal: self.normal.clone(),
                p: p,
                t: t,
            };
            return Some(h);
        }
        return None;
    }
}
/* }}} */
