use image::{
    Rgb,
};
use rand::Rng;
use crate::raytracer::{
    Ray,
    Hit,
};
use crate::maths::{
    EPSILON,
    Vec3,
    solve_3variable_system,
};
use std::f64::{
    self,
    consts::PI,
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
        Triangle::new_ref(&a, &b, &c, &color)
    }
    pub fn new_ref(a: &Vec3, b: &Vec3, c: &Vec3, color: &Rgb<u8>) -> Triangle {
        let a = a.new_clean();
        let b = b.new_clean();
        let c = c.new_clean();

        let ba = b.to(&a);
        let bc = b.to(&c);
        let mut normal = ba.cross_product(&bc);
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
            if w.x < 0.|| w.y < 0. || w.z < 0. {
                return None;
            }
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
/* Tetrahedron {{{ */

pub struct Tetrahedron {
    base: Triangle,
    side1: Triangle,
    side2: Triangle,
    side3: Triangle,
    color: Vec3,
}
impl Tetrahedron {
    pub fn new(top: Vec3, height: f64, width: f64, angle: f64, color: Rgb<u8>) -> Tetrahedron {
        let bottom_center = Vec3::new(top.x, top.y - height, top.z);
        let A = Vec3::new(
            bottom_center.x + width * (angle + 2. * PI / 3.).cos(),
            bottom_center.y,
            bottom_center.z + width * (angle + 2. * PI / 3.).sin()
        );
        let B = Vec3::new(
            bottom_center.x + width * (angle + 4. * PI / 3.).cos(),
            bottom_center.y,
            bottom_center.z + width * (angle + 4. * PI / 3.).sin()
        );
        let C = Vec3::new(
            bottom_center.x + width * (angle).cos(),
            bottom_center.y,
            bottom_center.z + width * (angle).sin()
        );
        Tetrahedron {
            base: Triangle::new_ref(&A, &C, &B, &color),
            side1: Triangle::new_ref(&A, &B, &top, &color),
            side2: Triangle::new_ref(&B, &C, &top, &color),
            side3: Triangle::new_ref(&C, &A, &top, &color),
            color: color.into(),
        }
    }
}
impl Object for Tetrahedron {
    fn hits(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<Hit> {
        let mut t_min = f64::INFINITY;
        let mut hit_min = None;
        for o in vec![&self.base, &self.side1, &self.side2, &self.side3] {
            if let Some(hit) = o.hits(&ray, 0_f64, t_min) {
                if hit.t < t_min  && hit.t >= tmin && hit.t <= tmax {
                    t_min = hit.t;
                    hit_min = Some(hit);
                }
            }
        }
        hit_min
    }
}
/* }}} */
/* Conifer {{{ */
pub struct Conifer {
    tetrahedrons: Vec<Tetrahedron>,
    color: Vec3,
}
const CONIFER_RATIO : f64 = 1.8;
impl Conifer {
    pub fn new(center: Vec3, base_width: f64, steps: u8) -> Conifer {
        let mut rng = rand::thread_rng();
        let mut angle = rng.gen::<f64>() * 2. * PI;
        let mut tetrahedrons = Vec::new();
        let mut width = base_width;
        let mut height = base_width * CONIFER_RATIO;
        let mut top = Vec3 {
            x: center.x,
            y: center.y + height,
            z: center.z
        };
        let color = Rgb([34, 139, 34]);
        for _ in 0..steps {
            let th = Tetrahedron::new(
                top.clone(),
                height, width, angle,
                color.clone()
                );
            // next loop
            top.y -= height * 0.6;
            angle += PI;
            width *= 0.8;
            height *= 0.8;
            top.y += height;

            tetrahedrons.push(th);
        }
        Conifer {
            tetrahedrons: tetrahedrons,
            color: color.into(),
        }
    }
}
impl Object for Conifer {
    fn hits(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<Hit> {
        let mut t_min = f64::INFINITY;
        let mut hit_min = None;
        for o in &self.tetrahedrons {
            if let Some(hit) = o.hits(&ray, 0_f64, t_min) {
                if hit.t < t_min  && hit.t >= tmin && hit.t <= tmax {
                    t_min = hit.t;
                    hit_min = Some(hit);
                }
            }
        }
        hit_min
    }
}
