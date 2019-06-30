use crate::maths::{Vec3, EPSILON};
use crate::object::{ObjectTrait, Plan};
use crate::scene::Scene;
use chrono::{DateTime, Local};
use color_scaling::scale_rgb;
use image::{ImageBuffer, Rgb, RgbImage};
use rand::Rng;
use rayon::prelude::*;
use std::f64;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub static DEPTH_MAX: u8 = 8;

#[derive(Debug)]
pub struct Hit {
    pub color: Vec3,
    pub normal: Vec3,
    pub p: Vec3,
    pub t: f64,
}

impl Hit {
    fn new() -> Hit {
        Hit {
            color: Vec3::new(0., 0., 0.),
            normal: Vec3::origin(),
            p: Vec3::origin(),
            t: f64::INFINITY,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Eye {
    pub origin: Vec3,
    pub direction: Vec3,
}

#[derive(Debug, Clone)]
pub struct Screen {
    pub width: u32,
    pub height: u32,
}
#[derive(Debug, Clone)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}
#[derive(Debug, Clone)]
pub struct Footprint {
    pub ne: Vec3,
    pub nw: Vec3,
    pub se: Vec3,
    pub sw: Vec3,
}
impl Footprint {
    pub fn get_real_position(&self, i: f64, j: f64) -> Vec3 {
        let e = self.se.mix(&self.ne, j);
        let w = self.sw.mix(&self.nw, j);
        w.mix(&e, i)
    }
    pub fn get_surface(&self) -> f64 {
        let south = self.sw.length_sq_to(&self.se).sqrt();
        let north = self.sw.length_sq_to(&self.se).sqrt();
        let avg_south = self.sw.avg(&self.se);
        let avg_north = self.nw.avg(&self.ne);
        let h = avg_south.length_sq_to(&avg_north).sqrt();
        h * (south + (north - south) / 2_f64)
    }
}

pub struct RayCtx {
    pub aspect_ratio: f64,
    pub eye: Eye,
    pub screen: Screen,
    pub w: Vec3,
    pub b: Vec3,
    pub v: Vec3,
    pub c: Vec3,
    pub p_top_left: Vec3,
    pub p_top_right: Vec3,
    pub p_bottom_left: Vec3,
    pub p_bottom_right: Vec3,
    pub width: f64,
    pub height: f64,
    pub hx: Vec3,
    pub hy: Vec3,
}
impl RayCtx {
    pub fn new(eye: &Eye, screen: &Screen) -> RayCtx {
        let w = Vec3::new_normalized(0., 1., 0.);
        let b = w.cross_product(&eye.direction).normalize(); // →
        let v = eye.direction.cross_product(&b).normalize(); // ↑
        debug!("b:{:?} v:{:?}", b, v);
        let d = 1.;
        let c = eye.origin.translate(&eye.direction, d);
        // Obtain the image's width and height.
        let width = screen.width as f64;
        let height = screen.height as f64;
        let aspect_ratio = width / height;
        debug!("eye:{:?} direction:{:?}", eye.origin, eye.direction);
        debug!("c:{:?}", c);

        /* Use 90° as horizontal field of view
         * Tan(π/4) = 1
         */

        let p_top_left = Vec3::new(
            c.x - b.x + v.x / aspect_ratio,
            c.y - b.y + v.y / aspect_ratio,
            c.z - b.z + v.z / aspect_ratio,
        );
        let p_top_right = Vec3::new(
            c.x + b.x + v.x / aspect_ratio,
            c.y + b.y + v.y / aspect_ratio,
            c.z + b.z + v.z / aspect_ratio,
        );
        let p_bottom_left = Vec3::new(
            c.x - b.x - v.x / aspect_ratio,
            c.y - b.y - v.y / aspect_ratio,
            c.z - b.z - v.z / aspect_ratio,
        );
        let p_bottom_right = Vec3::new(
            c.x + b.x - v.x / aspect_ratio,
            c.y + b.y - v.y / aspect_ratio,
            c.z + b.z - v.z / aspect_ratio,
        );
        debug!(
            "top_left:{:?} top_right:{:?} bottom_left:{:?} bottom_right:{:?}",
            p_top_left, p_top_right, p_bottom_left, p_bottom_right
        );
        let hx = p_top_left.to(&p_top_right);
        let hy = p_bottom_left.to(&p_top_left);

        debug!("hx:{:?}, hy:{:?}", hx, hy);

        RayCtx {
            aspect_ratio: aspect_ratio,
            eye: (*eye).clone(),
            screen: (*screen).clone(),
            w: w,
            b: b,
            v: v,
            c: c,
            p_top_left: p_top_left,
            p_top_right: p_top_right,
            p_bottom_right: p_bottom_right,
            p_bottom_left: p_bottom_left,
            width: width,
            height: height,
            hx: hx,
            hy: hy,
        }
    }

    pub fn get_footprint(&self, floor: &Plan) -> Footprint {
        let ft = |i, j| {
            let r = Ray::new(&self, i, j);
            let h = floor.hits(&r, 0_f64, f64::INFINITY);
            if let Some(hit) = h {
                hit.p
            } else {
                Vec3::infinity()
            }
        };
        Footprint {
            ne: ft(1.1, 1.1),
            nw: ft(-0.1, 1.1),
            se: ft(1.1, -0.1),
            sw: ft(-0.1, -0.1),
        }
    }

    pub fn render_scene(&self, scene: &Scene, nsamples: u64) -> RgbImage {
        let black: Rgb<u8> = Rgb([0, 0, 0]);
        let mut buf: Vec<Rgb<u8>> = vec![black; (self.screen.width * self.screen.height) as usize];
        let max_rays = (self.screen.width as u64) * (self.screen.height as u64) * nsamples;
        let nb_rays = Arc::new(AtomicUsize::new(1));
        let start: DateTime<Local> = Local::now();

        debug!("rendering scene");
        buf.par_iter_mut().enumerate().for_each(
            //buf.iter_mut().enumerate().for_each(
            |(n, pixel)| {
                let y = (n as u32) / self.screen.width;
                let x = (n as u32) - (y * self.screen.width);
                let i_min = x as f64 / self.width;
                let i_max = (x + 1) as f64 / self.width;
                let j_min = y as f64 / self.height;
                let j_max = (y + 1) as f64 / self.height;

                let i_step = i_max - i_min;
                let j_step = j_max - j_min;

                let mut r = 0_f64;
                let mut g = 0_f64;
                let mut b = 0_f64;

                let mut rng = rand::thread_rng();

                for _ in 0..nsamples {
                    let i = i_min + rng.gen::<f64>() * i_step;
                    let j = j_min + rng.gen::<f64>() * j_step;

                    let p = self.cast_ray_from_eye(scene, i, 1_f64 - j);
                    r += p.x;
                    g += p.y;
                    b += p.z;
                }

                r /= nsamples as f64;
                g /= nsamples as f64;
                b /= nsamples as f64;
                *pixel = Vec3::new(r, g, b).into();
                let nb_rays = nb_rays.fetch_add(nsamples as usize, Ordering::SeqCst);
                let now: DateTime<Local> = Local::now();
                let duration = now.signed_duration_since(start);
                let d_ms = duration.num_milliseconds();
                let end_ms = d_ms * (max_rays as i64) / (nb_rays as i64);
                let end_d = chrono::Duration::milliseconds(end_ms);
                let end = start.checked_add_signed(end_d).unwrap();

                print!(
                    "\r> {:>12} / {:} ({:3}%) end at {:?}",
                    nb_rays,
                    max_rays,
                    100_u64 * (nb_rays as u64) / (max_rays as u64),
                    end.to_rfc2822()
                );
            },
        );
        let mut img: RgbImage = ImageBuffer::new(self.screen.width, self.screen.height);
        for n in 0..(self.screen.width * self.screen.height) {
            let y = (n as u32) / self.screen.width;
            let x = (n as u32) - (y * self.screen.width);
            img.put_pixel(x, y, buf[n as usize]);
        }

        img
    }

    fn cast_ray_from_eye(&self, scene: &Scene, i: f64, j: f64) -> Vec3 {
        let r = Ray::new(&self, i, j);
        debug!("({:?},{:?}) r:{:?}", i, j, r);
        color(&r, scene, 0)
    }
}

impl Ray {
    /* i, j in [0,1], in usual direction (origin is bottom left) */
    fn new(ctx: &RayCtx, i: f64, j: f64) -> Ray {
        /* Origin is the eye */
        let screen_point = Vec3::new(
            ctx.p_bottom_left.x + i * ctx.hx.x + j * ctx.hy.x,
            ctx.p_bottom_left.y + i * ctx.hx.y + j * ctx.hy.y,
            ctx.p_bottom_left.z + i * ctx.hx.z + j * ctx.hy.z,
        );

        let mut d = ctx.eye.origin.to(&screen_point);
        d.normalize();
        let r = Ray {
            origin: ctx.eye.origin.clone(),
            direction: d,
        };
        r
    }

    pub fn at(&self, t: f64) -> Vec3 {
        self.direction.at(&self.origin, t)
    }
}

fn lambertian(hit: &Hit, scene: &Scene, depth: u8) -> Vec3 {
    let u = Vec3::random_in_unit_sphere();
    let lambertian = Ray {
        origin: hit.p.clone(),
        direction: u.addv(&hit.normal),
    };
    let c = color(&lambertian, &scene, depth + 1);
    c.multv(&hit.color)
}

fn hits(ray: &Ray, scene: &Scene) -> Hit {
    let mut hit_min = Hit::new();

    for o in &scene.objects {
        if let Some(hit) = o.hits(&ray, 0_f64, hit_min.t) {
            if hit.t < hit_min.t {
                hit_min = hit;
            }
        }
    }
    hit_min
}

fn color(ray: &Ray, scene: &Scene, depth: u8) -> Vec3 {
    let mut hit_min = hits(ray, scene);

    if hit_min.t == f64::INFINITY {
        /* ray hit the sky */
        let daylight = true;
        let c1: Rgb<u8>;
        let c2: Rgb<u8>;
        if daylight {
            /* daylight */
            c1 = Rgb([255, 255, 255]);
            c2 = Rgb([77, 143, 170]);
        } else {
            /* dusk */
            c1 = Rgb([20, 24, 42]);
            c2 = Rgb([43, 47, 82]);
        }
        let ud = ray.direction.normalize();
        //assert!(ud.y >= 0_f64);
        //assert!(ud.y <= 1_f64);
        scale_rgb(&c2, &c1, f64::abs(ud.y)).unwrap().into()
    } else {
        let with_lambertian = true;
        let with_shadows = true;
        let mut c: Vec3;
        if with_lambertian {
            if depth > DEPTH_MAX {
                return Vec3::new(0., 0., 0.);
            }
            c = lambertian(&hit_min, scene, depth);
        } else {
            c = hit_min.color;
        }
        if with_shadows {
            if let Some((ref sun, ref sun_color, ref softness)) = scene.sun {
                let start = hit_min.normal.at(&hit_min.p, EPSILON);
                let sun_ray = Ray {
                    origin: start,
                    direction: sun.clone(),
                };
                let sun_hit = hits(&sun_ray, scene);
                if sun_hit.t == f64::INFINITY {
                    c.mixed(sun_color, 1. - *softness);
                } else {
                    c.mult(*softness);
                }
            }
        }
        c
    }
}
