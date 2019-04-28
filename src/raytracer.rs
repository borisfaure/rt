use color_scaling::scale_rgb;
use image::{
    ImageBuffer,
    Rgb,
    RgbImage,
};
use crate::scene::{
    Scene,
};
use crate::maths::{
    EPSILON,
    Vec3,
};
use rand::Rng;
use std::f64;
use rayon::prelude::*;

pub static DEPTH_MAX : u8 = 8;

pub struct Hit {
    pub color: Vec3,
    pub normal : Vec3,
    pub p : Vec3,
    pub t : f64,
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

#[derive(Debug,Clone)]
pub struct Eye {
    pub origin: Vec3,
    pub direction: Vec3,
}

#[derive(Debug,Clone)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

struct RayCtx {
    eye: Eye,
    w: Vec3,
    b: Vec3,
    v: Vec3,
    c: Vec3,
    p_top_left: Vec3,
    p_top_right: Vec3,
    p_bottom_left: Vec3,
    width: f64,
    height: f64,
    hx: Vec3,
    hy: Vec3,
}
impl RayCtx {
    fn new(eye: &Eye, width: u32, height: u32) -> RayCtx {
        let w = Vec3::new(0., 1., 0.);
        let b = w.cross_product(&eye.direction); // →
        let v = eye.direction.cross_product(&b); // ↑
        debug!("b:{:?} v:{:?}", b, v);
        let d = 1.;
        let c = eye.origin.translate(&eye.direction, d);
        // Obtain the image's width and height.
        let width = width as f64;
        let height = height as f64;
        let aspect_ratio = width / height;
        debug!("eye:{:?} direction:{:?}", eye.origin, eye.direction);
        debug!("c:{:?}", c);

        /* Use 90° as horizontal field of view
         * Tan(π/4) = 1
         */

        let p_top_left = Vec3::new(
            c.x - b.x + v.x / aspect_ratio,
            c.y - b.y + v.y / aspect_ratio,
            c.z - b.z + v.z / aspect_ratio);
        let p_top_right = Vec3::new(
            c.x + b.x + v.x / aspect_ratio,
            c.y + b.y + v.y / aspect_ratio,
            c.z + b.z + v.z / aspect_ratio);
        let p_bottom_left = Vec3::new(
            c.x - b.x - v.x / aspect_ratio,
            c.y - b.y - v.y / aspect_ratio,
            c.z - b.z - v.z / aspect_ratio);
        debug!("top_left:{:?} top_right:{:?} bottom_left:{:?}",
               p_top_left, p_top_right, p_bottom_left);
        let hx = p_top_left.to(&p_top_right);
        let hy = p_bottom_left.to(&p_top_left);

        debug!("hx:{:?}, hy:{:?}", hx ,hy);

        RayCtx {
            eye: (*eye).clone(),
            w: w,
            b: b,
            v: v,
            c: c,
            p_top_left: p_top_left,
            p_top_right: p_top_right,
            p_bottom_left: p_bottom_left,
            width: width,
            height: height,
            hx: hx,
            hy: hy,
        }
    }
}

impl Ray {
    /* i, j in [0,1], in usual direction (origin is bottom left) */
    fn new(ctx: &RayCtx, i: f64, j: f64) -> Ray {
        /* Origin is the eye */
        let screen_point = Vec3::new(
            ctx.p_bottom_left.x
              + i * ctx.hx.x
              + j * ctx.hy.x,
            ctx.p_bottom_left.y
              + i * ctx.hx.y
              + j * ctx.hy.y,
            ctx.p_bottom_left.z
              + i * ctx.hx.z
              + j * ctx.hy.z);

        let mut d = ctx.eye.origin.to(&screen_point);
        d.normalize();
        let r = Ray {origin: ctx.eye.origin.clone(), direction: d};
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
    let c = color(&lambertian, &scene, depth+1);
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
        let white : Rgb<u8> = Rgb([255, 255, 255]);
        let blue  : Rgb<u8> = Rgb([ 77, 143, 170]);
        let ud = ray.direction.to_normalized();
        //assert!(ud.y >= 0_f64);
        //assert!(ud.y <= 1_f64);
        scale_rgb(&blue, &white, f64::abs(ud.y)).unwrap().into()
    } else {
        let with_lambertian = true;
        let with_shadows = true;
        let mut c : Vec3;
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
                    c.mix(sun_color, 1. - *softness);
                } else {
                    c.mult(*softness);
                }
            }
        }
        c
    }
}

fn cast_ray_from_eye(ctx: &RayCtx, scene: &Scene, i: f64, j: f64) -> Vec3 {
    let r = Ray::new(&ctx, i, j);
    debug!("({:?},{:?}) r:{:?}", i, j, r);
    assert!(r.direction.z >= 0.);
    color(&r, scene, 0)
}

pub fn render_scene(scene: &Scene, eye: &Eye, nsamples: u64, width: u32, height: u32) -> RgbImage {
    let ctx = RayCtx::new(&eye, width, height);
    let black : Rgb<u8> = Rgb([0, 0, 0]);
    let mut buf: Vec<Rgb<u8>> = vec![black; (width * height) as usize];

    debug!("rendering scene");
    //buf.par_iter_mut().enumerate().for_each(
    buf.iter_mut().enumerate().for_each(
        |(n, pixel)| {
            let y = (n as u32) / width;
            let x = (n as u32) - (y * width);
            let i_min = x as f64 / ctx.width;
            let i_max = (x+1) as f64 / ctx.width;
            let j_min = y as f64 / ctx.height;
            let j_max = (y+1) as f64 / ctx.height;

            let i_step = i_max - i_min;
            let j_step = j_max - j_min;

            let mut r = 0_f64;
            let mut g = 0_f64;
            let mut b = 0_f64;

            let mut rng = rand::thread_rng();

            for _ in 0..nsamples {
                let i = i_min + rng.gen::<f64>() * i_step;
                let j = j_min + rng.gen::<f64>() * j_step;

                let p = cast_ray_from_eye(&ctx, scene, i, 1_f64 - j);
                r += p.x;
                g += p.y;
                b += p.z;
            }

            r /= nsamples as f64;
            g /= nsamples as f64;
            b /= nsamples as f64;
            *pixel = Vec3::new(r,g,b).into()
        });
    let mut img : RgbImage = ImageBuffer::new(width, height);
    for n in 0..(width*height) {
        let y = (n as u32) / width;
        let x = (n as u32) - (y * width);
        img.put_pixel(x, y, buf[n as usize]);
    };

    img
}
