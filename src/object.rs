use color_scaling::scale_rgb;
use image::{
    Rgb,
};
use rand::Rng;
use crate::raytracer::{
    Ray,
    Hit,
    RayCtx,
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
#[derive(Clone)]
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
/* {{{ Ellipsoid */
pub struct Ellipsoid {
    center: Vec3,
    translation: Vec3,
    radii: Vec3,
    inv_radii: Vec3,
    sphere: Sphere,
}
impl Ellipsoid {
    pub fn new(center: Vec3, radii: Vec3, color: Rgb<u8>) -> Ellipsoid{
        let center = center;
        let translation = center.opposite();
        let inv_radii = radii.invert();
        let s = Sphere::new(Vec3::origin(), 1., color);
        Ellipsoid {
            center: center,
            translation: translation,
            radii: radii,
            inv_radii: inv_radii,
            sphere: s,
        }
    }
}

impl Object for Ellipsoid {
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
    bounding_sphere: Sphere,
    pub top: Vec3,
    pub height: f64,
    color: Vec3,
}
const CONIFER_RATIO : f64 = 1.8;
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
            z: base.z
        };
        let g1 = Rgb([0, 151, 0]);
        let g2 = Rgb([61, 159, 73]);
        let color = scale_rgb(&g1, &g2, rng.gen::<f64>()).unwrap();
        for i in 0..steps {
            let th = Tetrahedron::new(
                top.clone(),
                height, width, angle,
                color.clone()
                );
            // next loop
            if i != steps -1 {
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
            Vec3::new(
                base.x,
                base.y + (top.y - base.y) / 3.,
                base.z,
            ),
            (top.y - base.y) * 0.8,
            Rgb([0, 0, 0])
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
impl Object for Conifer {
    fn hits(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<Hit> {
        let mut t_min = f64::INFINITY;
        let mut hit_min = None;
        if let Some(_) = self.bounding_sphere.hits(&ray, 0_f64, t_min) {
            for o in &self.tetrahedrons {
                if let Some(hit) = o.hits(&ray, 0_f64, t_min) {
                    if hit.t < t_min  && hit.t >= tmin && hit.t <= tmax {
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
/* Owl {{{ */
pub struct Owl {
    pub objects: Vec<Box<Object + Sync + Send>>,
    pub bounding_sphere: Sphere,
}
impl Owl {
    pub fn new(base: Vec3, height: f64) -> Owl {
        let mut objs: Vec<Box<Object + Sync + Send>> = Vec::new();
        let brown = Rgb([98, 57, 17]);
        let orange = Rgb([242, 123, 8]);
        let black = Rgb([2, 2, 2]);
        let body_height = height * 0.7;
        let head_radius = body_height / 4.;
        let body = Ellipsoid::new(
            base.clone(),
            Vec3::new(head_radius * 1.2,
                      body_height / 2.,
                      head_radius * 1.2),
            brown.clone(),
            );
        let head = Sphere::new(
            Vec3::new(base.x,
                      base.y + height * 0.4 + head_radius / 2.,
                      base.z),
            head_radius,
            brown.clone(),
            );
        let eye_y = base.y + height * 0.4 + head_radius * 1.2;
        let right_eye = Sphere::new(
            Vec3::new(
                base.x + head_radius * 0.4,
                eye_y,
                base.z - head_radius * 0.55),
            head_radius * 0.1,
            black.clone()
            );
        let left_eye = Sphere::new(
            Vec3::new(
                base.x - head_radius * 0.4,
                eye_y,
                base.z - head_radius * 0.55),
            head_radius * 0.1,
            black.clone()
            );
        let beak = Ellipsoid::new(
            Vec3::new(
                base.x,
                base.y + height * 0.4 + head_radius,
                base.z - head_radius * 0.9
                ),
            Vec3::new(head_radius * 0.1,
                      head_radius * 0.2,
                      head_radius * 0.1),
            black.clone()
            );
        let ear_y = base.y + height * 0.4 + head_radius * 2.2;
        let left_ear = Tetrahedron::new(
            Vec3::new(
                base.x - head_radius * 0.5,
                ear_y,
                base.z
            ),
            head_radius * 2.,
            head_radius * 0.4,
            0.,
            brown.clone()
            );
        let right_ear = Tetrahedron::new(
            Vec3::new(
                base.x + head_radius * 0.5,
                ear_y,
                base.z
            ),
            head_radius * 2.,
            head_radius * 0.4,
            0.,
            brown.clone()
            );
        let bs = Sphere::new(
            Vec3::new(
                base.x,
                base.y + height / 2.,
                base.z,
            ),
            height * 0.7,
            Rgb([0, 0, 0])
            );

        objs.push(Box::new(body));
        objs.push(Box::new(head));
        objs.push(Box::new(right_eye));
        objs.push(Box::new(left_eye));
        objs.push(Box::new(beak));
        objs.push(Box::new(left_ear));
        objs.push(Box::new(right_ear));
        Owl {
            objects: objs,
            bounding_sphere: bs,
        }
    }
}
impl Object for Owl {
    fn hits(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<Hit> {
        let mut t_min = f64::INFINITY;
        let mut hit_min = None;
        if let Some(_) = self.bounding_sphere.hits(&ray, 0_f64, t_min) {
            for o in &self.objects {
                if let Some(hit) = o.hits(&ray, 0_f64, t_min) {
                    if hit.t < t_min  && hit.t >= tmin && hit.t <= tmax {
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
/* Signature {{{ */
pub struct Signature {
    pub objects: Vec<Box<Object + Sync + Send>>,
    pub bounding_sphere: Sphere,
}
impl Signature {
    pub fn new(ray_ctx: &RayCtx) -> Signature {
        let mut objs: Vec<Box<Object + Sync + Send>> = Vec::new();
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
            objs.push(Box::new(sphere));
        };
        let bs = Sphere::new(
            Vec3::new(
                base.x + 12.5 * diameter * ray_ctx.b.x
                    + 2.5 * diameter * ray_ctx.v.x,
                base.y + 12.5 * diameter * ray_ctx.b.y
                    + 2.5 * diameter * ray_ctx.v.y,
                base.z + 12.5 * diameter * ray_ctx.b.z
                    + 2.5 * diameter * ray_ctx.v.z,
            ),
            16. * diameter,
            Rgb([0, 0, 0])
            );
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

        Signature {
            objects: objs,
            bounding_sphere: bs,
        }
    }
}
impl Object for Signature {
    fn hits(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<Hit> {
        let mut t_min = f64::INFINITY;
        let mut hit_min = None;
        if let Some(_) = self.bounding_sphere.hits(&ray, 0_f64, t_min) {
            for o in &self.objects {
                if let Some(hit) = o.hits(&ray, 0_f64, t_min) {
                    if hit.t < t_min  && hit.t >= tmin && hit.t <= tmax {
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
