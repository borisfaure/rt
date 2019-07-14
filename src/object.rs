use crate::maths::{solve_3variable_system, Vec3, EPSILON};
use crate::raytracer::{Hit, Ray};
use color_scaling::scale_rgb;
use image::Rgb;
use rand::Rng;
use std::f64::{self, consts::PI};

pub trait ObjectTrait {
    fn hits(&self, r: &Ray, tmin: f64, tmax: f64) -> Option<Hit>;
}

/* {{{ Plan */
#[derive(Clone, Serialize, Deserialize)]
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
impl ObjectTrait for Plan {
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
#[derive(Clone, Serialize, Deserialize)]
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
            color: color.into(),
        }
    }
}

impl ObjectTrait for Sphere {
    fn hits(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<Hit> {
        let oc = self.center.to(&ray.origin);
        let a = ray.direction.dot_product(&ray.direction);
        let b = oc.dot_product(&ray.direction);
        let c = oc.dot_product(&oc) - self.rd_sq;
        let discrimant = b * b - a * c;
        if discrimant <= 0_f64 {
            return None;
        }
        let discrimant_sqrt = discrimant.sqrt();
        let t1 = (-b - discrimant_sqrt) / a;
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
        let t2 = (-b + discrimant_sqrt) / a;
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
/* {{{ Ellipsoid */
#[derive(Clone, Serialize, Deserialize)]
pub struct Ellipsoid {
    center: Vec3,
    translation: Vec3,
    radii: Vec3,
    inv_radii: Vec3,
    sphere: Sphere,
}
impl Ellipsoid {
    //    pub fn new(center: Vec3, radii: Vec3, color: Rgb<u8>) -> Ellipsoid {
    //        let center = center;
    //        let translation = center.opposite();
    //        let inv_radii = radii.invert();
    //        let s = Sphere::new(Vec3::origin(), 1., color);
    //        Ellipsoid {
    //            center: center,
    //            translation: translation,
    //            radii: radii,
    //            inv_radii: inv_radii,
    //            sphere: s,
    //        }
    //    }
}

impl ObjectTrait for Ellipsoid {
    fn hits(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<Hit> {
        let ray2 = Ray {
            origin: ray.origin.addv(&self.translation).multv(&self.inv_radii),
            direction: ray.direction.multv(&self.inv_radii),
        };
        let h = self.sphere.hits(&ray2, tmin, tmax);
        if let Some(hit) = h {
            let n = hit.normal;
            let p = hit.p.multv(&self.radii).addv(&self.center);
            let h = Hit {
                color: hit.color,
                normal: n,
                p: p,
                t: hit.t,
            };
            Some(h)
        } else {
            None
        }
    }
}

/* }}} */
/* Triangle {{{ */

#[derive(Clone, Serialize, Deserialize)]
pub struct Triangle {
    a: Vec3,
    b: Vec3,
    c: Vec3,
    normal: Vec3,
    color: Vec3,
}
impl Triangle {
    pub fn new_ref(a: &Vec3, b: &Vec3, c: &Vec3, color: &Rgb<u8>) -> Triangle {
        let a = a.new_clean();
        let b = b.new_clean();
        let c = c.new_clean();

        let ba = b.to(&a);
        let bc = b.to(&c);
        let normal = ba.cross_product(&bc).normalize();
        Triangle {
            a: a,
            b: b,
            c: c,
            normal: normal,
            color: color.into(),
        }
    }
}
impl ObjectTrait for Triangle {
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

#[derive(Clone, Serialize, Deserialize)]
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
        let a = Vec3::new(
            bottom_center.x + width * (angle + 2. * PI / 3.).cos(),
            bottom_center.y,
            bottom_center.z + width * (angle + 2. * PI / 3.).sin(),
        );
        let b = Vec3::new(
            bottom_center.x + width * (angle + 4. * PI / 3.).cos(),
            bottom_center.y,
            bottom_center.z + width * (angle + 4. * PI / 3.).sin(),
        );
        let c = Vec3::new(
            bottom_center.x + width * (angle).cos(),
            bottom_center.y,
            bottom_center.z + width * (angle).sin(),
        );
        Tetrahedron {
            base: Triangle::new_ref(&a, &c, &b, &color),
            side1: Triangle::new_ref(&a, &b, &top, &color),
            side2: Triangle::new_ref(&b, &c, &top, &color),
            side3: Triangle::new_ref(&c, &a, &top, &color),
            color: color.into(),
        }
    }
}
impl ObjectTrait for Tetrahedron {
    fn hits(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<Hit> {
        let mut t_min = f64::INFINITY;
        let mut hit_min = None;
        for o in vec![&self.base, &self.side1, &self.side2, &self.side3] {
            if let Some(hit) = o.hits(&ray, 0_f64, t_min) {
                if hit.t < t_min && hit.t >= tmin && hit.t <= tmax {
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
#[derive(Clone, Serialize, Deserialize)]
pub struct Conifer {
    tetrahedrons: Vec<Tetrahedron>,
    bounding_sphere: Sphere,
    pub top: Vec3,
    pub height: f64,
    color: Vec3,
}
const CONIFER_RATIO: f64 = 1.8;
impl Conifer {
    pub fn new(base: Vec3, base_width: f64, steps: u8) -> Conifer {
        let mut rng = rand::thread_rng();
        let mut angle = rng.gen::<f64>() * 2. * PI;
        let mut tetrahedrons = Vec::new();
        let mut width = base_width;
        let mut height = base_width * CONIFER_RATIO;
        let mut top = Vec3 {
            x: base.x,
            y: base.y + height,
            z: base.z,
        };
        let g1 = Rgb([0, 151, 0]);
        let g2 = Rgb([61, 159, 73]);
        let color = scale_rgb(&g1, &g2, rng.gen::<f64>()).unwrap();
        for i in 0..steps {
            let th = Tetrahedron::new(top.clone(), height, width, angle, color.clone());
            // next loop
            if i != steps - 1 {
                top.y -= height * 0.6;
                angle += PI;
                width *= 0.8;
                height *= 0.8;
                top.y += height;
                assert!(top.y - height >= base.y);
            }

            tetrahedrons.push(th);
        }
        let bs = Sphere::new(
            Vec3::new(base.x, base.y + (top.y - base.y) / 3., base.z),
            (top.y - base.y) * 0.8,
            Rgb([0, 0, 0]),
        );
        let height = top.y - base.y;
        Conifer {
            tetrahedrons: tetrahedrons,
            bounding_sphere: bs,
            top: top,
            height: height,
            color: color.into(),
        }
    }
}
impl ObjectTrait for Conifer {
    fn hits(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<Hit> {
        let mut t_min = f64::INFINITY;
        let mut hit_min = None;
        if let Some(_) = self.bounding_sphere.hits(&ray, 0_f64, t_min) {
            for o in &self.tetrahedrons {
                if let Some(hit) = o.hits(&ray, 0_f64, t_min) {
                    if hit.t < t_min && hit.t >= tmin && hit.t <= tmax {
                        t_min = hit.t;
                        hit_min = Some(hit);
                    }
                }
            }
        }
        hit_min
    }
}
/* }}} */

#[derive(Clone, Serialize, Deserialize)]
pub enum BaseObject {
    Plan(Plan),
    Sphere(Sphere),
    Ellipsoid(Ellipsoid),
    Triangle(Triangle),
    Tetrahedron(Tetrahedron),
    Conifer(Conifer),
}
impl ObjectTrait for BaseObject {
    fn hits(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<Hit> {
        match self {
            BaseObject::Plan(p) => p.hits(ray, tmin, tmax),
            BaseObject::Sphere(s) => s.hits(ray, tmin, tmax),
            BaseObject::Ellipsoid(e) => e.hits(ray, tmin, tmax),
            BaseObject::Triangle(t) => t.hits(ray, tmin, tmax),
            BaseObject::Tetrahedron(t) => t.hits(ray, tmin, tmax),
            BaseObject::Conifer(c) => c.hits(ray, tmin, tmax),
        }
    }
}
