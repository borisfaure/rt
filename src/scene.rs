use crate::maths::Vec3;
use crate::object::{BaseObject, Conifer, ObjectTrait, Sphere};
use crate::raytracer::{Footprint, Ray, RayCtx};
use image::{Rgb, RgbImage};
use rand::Rng;
use std::error::Error;
use std::f64::{self, consts::PI};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use rayon::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct Scene {
    pub objects: Vec<BaseObject>,
    pub sun: Option<(Vec3, Vec3, f64)>,
}

/* {{{ Circle */
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
/* }}} */

impl Scene {
    pub fn new() -> Scene {
        Scene {
            objects: Vec::new(),
            sun: None,
        }
    }
    pub fn add(&mut self, obj: BaseObject) {
        self.objects.push(obj);
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
    pub fn load(json_file_path: &Path) -> Scene {
        let f = match File::open(&json_file_path) {
            Err(why) => {
                let display = json_file_path.display();
                panic!("couldn't open {}: {}", display, why.description())
            }
            Ok(file) => file,
        };
        let reader = BufReader::new(f);
        match serde_json::from_reader(reader) {
            Err(why) => {
                let display = json_file_path.display();
                panic!("couldn't open {}: {}", display, why.description())
            }
            Ok(s) => s,
        }
    }
    pub fn save(&self, json_file_path: &Path) {
        let f = match File::create(&json_file_path) {
            Err(why) => {
                let display = json_file_path.display();
                panic!("couldn't create {}: {}", display, why.description())
            }
            Ok(file) => file,
        };
        if let Err(why) = serde_json::to_writer_pretty(f, self) {
            let display = json_file_path.display();
            panic!("couldn't create {}: {}", display, why.description())
        }
    }

    pub fn generate_from_image(&mut self, ray_ctx: &RayCtx, buf: RgbImage, nb_vert_spheres: f64) -> usize {
        let black = Rgb([0, 0, 0]);
        let mut nb_spheres: usize = 0;
        let radius = 2. * PI / (4. * nb_vert_spheres * ray_ctx.aspect_ratio - PI);
        let f = 2. + radius;
        info!("radius:{:?} f:{:?}", radius, f);
        let nb_horiz_spheres = (nb_vert_spheres * ray_ctx.aspect_ratio).ceil();
        let nb_vert_spheres = nb_vert_spheres + 1.;
        info!("nb_horiz_spheres:{:?} nb_vert_spheres:{:?}",
              nb_horiz_spheres, nb_vert_spheres);
        let rj = 1. / nb_vert_spheres / 2.;
        let ri = 1. / nb_horiz_spheres / 2.;

        let get_color_avg = |s: &Sphere, x: u32, y: u32| -> (Rgb<u8>, u64) {
            if x >= buf.width() || y >= buf.height() {
                return (black.clone(), 0);
            }
            let p = buf.get_pixel(x, buf.height() - 1 - y);
            let mut avg = 0;
            let t = |di, dj| {
                let i = ((x as f64) + di) / (buf.width() as f64);
                let j = ((y as f64) + dj) / (buf.height() as f64);
                let r = Ray::new(ray_ctx, i, j);
                let h = s.hits(&r, 0_f64, f64::INFINITY);
                if let Some(_) = h {
                    1
                } else {
                    0
                }
            };
            avg += t(0.25, 0.25);
            avg += t(0.25, 0.75);
            avg += t(0.75, 0.25);
            avg += t(0.75, 0.75);
            (p.clone(), avg)
        };
        let get_screenprint = |ci: f64, cj: f64| -> ((u32, u32), (u32, u32)) {
            let x0f = ((ci - ri) * (ray_ctx.width as f64)).trunc();
            let x0 = if x0f < 0. {
                0
            } else if x0f >= ray_ctx.width {
                ray_ctx.width as u32 - 1
            } else {
                x0f as u32
            };

            let y0f = ((cj - rj) * (ray_ctx.height as f64)).trunc();
            let y0 = if y0f < 0. {
                0
            } else if y0f >= ray_ctx.height {
                ray_ctx.height as u32 - 1
            } else {
                y0f as u32
            };

            let x1f = ((ci + ri) * (ray_ctx.width as f64)).ceil();
            let x1 = if x1f < 0. {
                0
            } else if x1f > ray_ctx.width {
                ray_ctx.width as u32
            } else {
                x1f as u32
            };

            let y1f = ((cj + rj) * (ray_ctx.height as f64)).ceil();
            let y1 = if y1f <= 0. {
                0
            } else if y1f > ray_ctx.height {
                ray_ctx.height as u32
            } else {
                y1f as u32
            };

            ((x0, y0), (x1, y1))
        };

        let get_color = |s: &Sphere, i: f64, j: f64| -> Rgb<u8> {
            let ((x0, y0), (x1, y1)) = get_screenprint(i, j);

            let mut vec: Vec<(Rgb<u8>, u64)> = Vec::new();
            for y in y0..y1 {
                for x in x0..x1 {
                    let (p, w) = get_color_avg(s, x, y);
                    if w > 0 {
                        vec.push((p, w));
                    }
                }
            }
            let mut w = 0_u64;
            let mut r = 0_u64;
            let mut g = 0_u64;
            let mut b = 0_u64;

            for (c, avg) in vec {
                r += (c[0] as u64) * avg;
                g += (c[1] as u64) * avg;
                b += (c[2] as u64) * avg;
                w += avg;
            }
            if w > 0 {
                Rgb([(r / w) as u8, (g / w) as u8, (b / w) as u8])
            } else {
                let mut rng = rand::thread_rng();
                Rgb([rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>()])
            }
        };
        let add_point = |i: f64, j: f64, is_front: bool| {
            /* Vertical fov is π/2 */
            let vangle = PI / 2. * (j - 0.5) / ray_ctx.aspect_ratio;
            let vsin = vangle.sin();
            let vcos = vangle.cos();
            let hangle = PI / 2. * (i - 0.5);
            let hsin = hangle.sin();
            let hcos = hangle.cos();

            let v = Vec3::new_normalized(
                vsin * ray_ctx.v.x + vcos * ray_ctx.eye.direction.x,
                vsin * ray_ctx.v.y + vcos * ray_ctx.eye.direction.y,
                vsin * ray_ctx.v.z + vcos * ray_ctx.eye.direction.z,
                );
            let h = Vec3::new_normalized(
                hsin * ray_ctx.b.x + hcos * ray_ctx.eye.direction.x,
                hsin * ray_ctx.b.y + hcos * ray_ctx.eye.direction.y,
                hsin * ray_ctx.b.z + hcos * ray_ctx.eye.direction.z,
                );
            let dir = v.addv(&h).subv(&ray_ctx.eye.direction).normalize();
            let (f,r) = if is_front {
                (f, radius)
            } else {
                (f + radius * 1.1, radius * 1.1)
            };
            let c = dir.at(&ray_ctx.eye.origin, f);
            let s = Sphere::new(c.clone(), r, black.clone(), true);
            let color = get_color(&s, i, j);
            Sphere::new(c, r, color.clone(), true)
        };

        let n = (nb_horiz_spheres * nb_vert_spheres) as u32;
        let spheres : Vec<(Sphere, Sphere)> = (0..n).into_par_iter().map(|idx| {
            //let spheres : Vec<Sphere> = (0..n).map(|idx| {
            let y = idx / (nb_horiz_spheres as u32);
            let x = idx % (nb_horiz_spheres as u32);
            let fi = (x as f64) / (nb_horiz_spheres - 1.);
            let fj = (y as f64) / (nb_vert_spheres - 1.);
            let f = add_point(fi, fj, true);
            let s = add_point(fi + ri, fj + rj, false);
            (f,s)
        }).collect();
        for (f,b) in spheres {
            self.add(BaseObject::Sphere(f));
            self.add(BaseObject::Sphere(b));
            nb_spheres += 2;
        }

        return nb_spheres;
    }

    pub fn add_signature(&mut self, ray_ctx: &RayCtx) {
        /* compute radius + bottom left pos */
        let p_bottom_right = ray_ctx.ij_to_screen(1., 1.);
        let p_top_right = ray_ctx.ij_to_screen(1., 0.);
        let diameter = 0.008
            * p_bottom_right
                .length_sq_to(&p_top_right)
                .sqrt();
        let radius = diameter / 2.;
        let c = ray_ctx
            .eye
            .origin
            .translate(&ray_ctx.eye.direction, 1. + 2. * diameter);
        let bottom_right = Vec3::new(
            c.x + ray_ctx.b.x - ray_ctx.v.x / ray_ctx.aspect_ratio,
            c.y + ray_ctx.b.y - ray_ctx.v.y / ray_ctx.aspect_ratio,
            c.z + ray_ctx.b.z - ray_ctx.v.z / ray_ctx.aspect_ratio,
        );
        let base = Vec3::new(
            bottom_right.x - 25. * diameter * ray_ctx.b.x + 2. * diameter * ray_ctx.v.x,
            bottom_right.y - 25. * diameter * ray_ctx.b.y + 2. * diameter * ray_ctx.v.y,
            bottom_right.z - 25. * diameter * ray_ctx.b.z + 2. * diameter * ray_ctx.v.z,
        );
        let color = Rgb([254, 55, 32]);
        let mut add_point = |x: f64, y: f64| {
            let v = Vec3::new(
                base.x + x * diameter * ray_ctx.b.x + y * diameter * ray_ctx.v.x,
                base.y + x * diameter * ray_ctx.b.y + y * diameter * ray_ctx.v.y,
                base.z + x * diameter * ray_ctx.b.z + y * diameter * ray_ctx.v.z,
            );
            let sphere = Sphere::new(v, radius, color.clone());
            self.add(BaseObject::Sphere(sphere));
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
        add_point(1.8, 1.);
        add_point(1.8, 3.);
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
        add_point(19.8, 0.);
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

    pub fn generate_forest_monte_carlo(&mut self, footprint: &Footprint, threshold: f64) -> u32 {
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
                self.add(BaseObject::Conifer(conifer));
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
