use crate::maths::{Vec3, EPSILON};
use crate::object::{ObjectTrait, Plan, Sphere};
use crate::scene::Scene;
use chrono::{DateTime, Local};
use color_scaling::scale_rgb;
use image::{Rgb, Rgba, RgbaImage};
use rand::Rng;
use rayon::prelude::*;
use std::f64;
use std::f64::consts::PI;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
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
#[derive(Debug)]
pub struct RayCtx {
    pub aspect_ratio: f64,
    pub eye: Eye,
    pub screen: Screen,
    pub w: Vec3,
    pub b: Vec3,
    pub v: Vec3,
    pub c: Vec3,
    pub width: f64,
    pub height: f64,
    pub with_lambertian: bool,
    pub with_shadows: bool,
}
#[derive(Debug, Clone)]
pub struct Ray<'a> {
    pub origin: Vec3,
    pub direction: Vec3,
    pub is_light: bool,
    pub ray_ctx: &'a RayCtx,
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

impl RayCtx {
    pub fn new(eye: &Eye, screen: &Screen, with_lambertian: bool, with_shadows: bool) -> RayCtx {
        let w = Vec3::new_normalized(0., 1., 0.);
        let b = w.cross_product(&eye.direction).normalize(); // →
        let v = eye.direction.cross_product(&b).normalize(); // ↑
        info!(
            "w:{:?} b:{:?} v:{:?} eye:{:?}",
            w, b, v, eye,
        );
        let d = 1.;
        let c = eye.origin.translate(&eye.direction, d);
        // Obtain the image's width and height.
        let width = screen.width as f64;
        let height = screen.height as f64;
        let aspect_ratio = width / height;

        let r = RayCtx {
            aspect_ratio: aspect_ratio,
            eye: (*eye).clone(),
            screen: (*screen).clone(),
            w: w,
            b: b,
            v: v,
            c: c,
            width: width,
            height: height,
            with_lambertian: with_lambertian,
            with_shadows: with_shadows,
        };
        info!("TL:{:?} TR:{:?} BR:{:?} BL:{:?}",
              r.ij_to_screen(0., 0.),
              r.ij_to_screen(1., 0.),
              r.ij_to_screen(1., 1.),
              r.ij_to_screen(0., 1.),
              );
        info!("T:{:?} B:{:?}",
              r.ij_to_screen(0.5, 0.),
              r.ij_to_screen(0.5, 1.),
              );
        r
    }

    pub fn ij_to_screen(&self, i: f64, j: f64) -> Vec3 {
        let i = i - 0.5;
        let j = j - 0.5;
        /* Horizontal fov is π/2 */
        let vangle = PI / 2. * j / self.aspect_ratio;
        let vsin = vangle.sin();
        let vcos = vangle.cos();
        let hangle = PI / 2. * i;
        let hsin = hangle.sin();
        let hcos = hangle.cos();

        let v = Vec3::new_normalized(
            vsin * self.v.x + vcos * self.eye.direction.x,
            vsin * self.v.y + vcos * self.eye.direction.y,
            vsin * self.v.z + vcos * self.eye.direction.z,
        );
        let h = Vec3::new_normalized(
            hsin * self.b.x + hcos * self.eye.direction.x,
            hsin * self.b.y + hcos * self.eye.direction.y,
            hsin * self.b.z + hcos * self.eye.direction.z,
        );

        let dir = v.addv(&h).subv(&self.eye.direction);
        self.eye.origin.addv(&dir)
    }

    pub fn get_footprint(&self, floor: &Plan) -> Footprint {
        let ft = |i, j| {
            let r = Ray::new(&self, i, j, false);
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

    pub fn render_scene(&self, scene: &Scene, nsamples: u64, pngpath: &str) {
        let mut buf: RgbaImage;
        let undone_color = Rgba([255u8, 0u8, 255u8, 0u8]);
        if let Ok(img) = image::open(pngpath) {
            buf = img.as_rgba8().unwrap().clone();
            assert!(buf.height() == self.screen.height);
            assert!(buf.width() == self.screen.width);
        } else {
            buf = image::RgbaImage::new(self.screen.width as u32, self.screen.height as u32);
            for (_, _, pixel) in buf.enumerate_pixels_mut() {
                *pixel = undone_color;
            }
        }
        let max_rays = (self.screen.width as u64) * (self.screen.height as u64) * nsamples;
        let nb_pix = Arc::new(AtomicUsize::new(0));
        let nb_pix_worked = Arc::new(AtomicUsize::new(0));
        let nb_pix_to_see: usize = (self.screen.height * self.screen.width) as usize;
        let start: DateTime<Local> = Local::now();

        let stop = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(signal_hook::SIGINT, Arc::clone(&stop)).ok();
        signal_hook::flag::register(signal_hook::SIGTERM, Arc::clone(&stop)).ok();

        dbg!("rendering scene");
        buf.enumerate_pixels_mut()
            .collect::<Vec<(u32, u32, &mut Rgba<u8>)>>()
            .par_iter_mut()
            .for_each(|(x, y, pixel)| {
                let stop = stop.load(Ordering::SeqCst);
                if stop {
                    return;
                }
                let worked;
                if **pixel == undone_color {
                    let i_min = *x as f64 / self.width;
                    let i_max = (*x + 1) as f64 / self.width;
                    let j_min = *y as f64 / self.height;
                    let j_max = (*y + 1) as f64 / self.height;

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
                    **pixel = Vec3::new(r, g, b).into();
                    worked = true;
                } else {
                    worked = false;
                }
                let pix_worked;
                let nb_pix = nb_pix.fetch_add(1 as usize, Ordering::SeqCst);
                if worked {
                    pix_worked = nb_pix_worked.fetch_add(1 as usize, Ordering::SeqCst);
                } else {
                    pix_worked = nb_pix_worked.load(Ordering::SeqCst);
                }
                let skipped;
                if pix_worked > 0 && nb_pix > pix_worked {
                    skipped = nb_pix - pix_worked;
                } else {
                    skipped = 0_usize;
                }

                let now: DateTime<Local> = Local::now();
                let duration = now.signed_duration_since(start);
                let d_ms = duration.num_milliseconds();
                let end_ms;
                if pix_worked > 0 {
                    end_ms = d_ms * ((nb_pix_to_see - skipped) as i64) / (pix_worked as i64);
                } else {
                    end_ms = d_ms * ((nb_pix_to_see - skipped) as i64);
                }
                let end_d = chrono::Duration::milliseconds(end_ms);
                let end = start.checked_add_signed(end_d).unwrap();

                print!(
                    "\r> {:>12} / {:} ({:3}%) end at {:?}",
                    nb_pix * nsamples as usize,
                    max_rays,
                    100_u64 * (nb_pix as u64) / (nb_pix_to_see as u64),
                    end.to_rfc2822(),
                );
            });

        buf.save(pngpath).ok();
    }

    fn cast_ray_from_eye(&self, scene: &Scene, i: f64, j: f64) -> Vec3 {
        let r = Ray::new(&self, i, j, false);
        r.color(scene, 0)
    }
}

impl<'a> Ray<'a> {
    /* i, j in [0,1], in usual direction (origin is bottom left) */
    pub fn new(ctx: &'a RayCtx, i: f64, j: f64, is_light: bool) -> Ray<'a> {
        /* Origin is the eye */
        let screen_point = ctx.ij_to_screen(i, j);
        let d = ctx.eye.origin.to(&screen_point).normalize();
        let r = Ray {
            origin: ctx.eye.origin.clone(),
            direction: d,
            is_light: is_light,
            ray_ctx: ctx,
        };
        r
    }

    pub fn at(&self, t: f64) -> Vec3 {
        self.direction.at(&self.origin, t)
    }

    fn hits(&'a self, scene: &Scene) -> Hit {
        let mut hit_min = Hit::new();

        for o in &scene.objects {
            if let Some(hit) = o.hits(&self, 0_f64, hit_min.t) {
                if hit.t < hit_min.t {
                    hit_min = hit;
                }
            }
        }
        hit_min
    }

    fn color(&'a self, scene: &Scene, depth: u8) -> Vec3 {
        let hit_min = self.hits(scene);

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
            let ud = self.direction.normalize();
            //assert!(ud.y >= 0_f64);
            //assert!(ud.y <= 1_f64);
            scale_rgb(&c2, &c1, f64::abs(ud.y)).unwrap().into()
        } else {
            let mut c: Vec3;
            if self.ray_ctx.with_lambertian {
                if depth > DEPTH_MAX {
                    return Vec3::new(0., 0., 0.);
                }
                let lambertian = |hit: &Hit, depth: u8| -> Vec3 {
                    let u = Vec3::random_in_unit_sphere();
                    let lambertian = Ray {
                        origin: hit.p.clone(),
                        direction: u.addv(&hit.normal),
                        is_light: false,
                        ray_ctx: self.ray_ctx,
                    };
                    let c = lambertian.color(&scene, depth + 1);
                    c.multv(&hit.color)
                };
                c = lambertian(&hit_min, depth);
            } else {
                c = hit_min.color;
            }
            if self.ray_ctx.with_shadows {
                if let Some((ref sun, ref sun_color, ref softness)) = scene.sun {
                    let start = hit_min.normal.at(&hit_min.p, EPSILON);
                    let sun_ray = Ray {
                        origin: start,
                        direction: sun.clone(),
                        is_light: true,
                        ray_ctx: self.ray_ctx,
                    };
                    let sun_hit = sun_ray.hits(scene);
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
}
