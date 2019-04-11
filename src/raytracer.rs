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
    Vec3,
};
use rand::Rng;
use std::f64;
use rayon::prelude::*;

pub struct Hit {
    pub color: Rgb<u8>,
    pub normal : Vec3,
    pub t : f64,
}

impl Hit {
    fn new() -> Hit {
        Hit {
            color: Rgb([0, 0, 0]),
            normal: Vec3::new(0., 0., 0.),
            t: 0.
        }
    }
    fn to_pixel(&self, _distance: f64) -> Rgb<u8> {
        /* TODO: */
        self.color
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
    hx: f64,
    hy: f64,
    qx: Vec3,
    qy: Vec3,
}
impl RayCtx {
    fn new(eye: &Eye, width: u32, height: u32) -> RayCtx {
        let w = Vec3::new(0., 1., 0.);
        let b = w.cross_product(&eye.direction);
        let v = eye.direction.cross_product(&b);
        debug!("b:{:?} v:{:?}", b, v);
        let d = 1.;
        let c = eye.origin.translate(&eye.direction, d);
        // Obtain the image's width and height.
        let width = width as f64;
        let height = height as f64;
        let aspect_ratio = width / height;

        /* Use 90° as horizontal field of view
         * Tan(π/4) = 1
         */
        let hx = d;
        let hy = hx / aspect_ratio;

        let p_top_left = Vec3::new(
            c.x - 0.5 * aspect_ratio * b.x + 0.5 * v.x,
            c.y - 0.5 * aspect_ratio * b.y + 0.5 * v.y,
            c.z - 0.5 * aspect_ratio * b.z + 0.5 * v.z);
        let p_top_right = Vec3::new(
            c.x + 0.5 * aspect_ratio * b.x + 0.5 * v.x,
            c.y + 0.5 * aspect_ratio * b.y + 0.5 * v.y,
            c.z + 0.5 * aspect_ratio * b.z + 0.5 * v.z);
        let p_bottom_left = Vec3::new(
            c.x - 0.5 * aspect_ratio * b.x - 0.5 * v.x,
            c.y - 0.5 * aspect_ratio * b.y - 0.5 * v.y,
            c.z - 0.5 * aspect_ratio * b.z - 0.5 * v.z);
        debug!("top_left:{:?} top_right:{:?} bottom_left:{:?}",
               p_top_left, p_top_right, p_bottom_left);
        let qx = Vec3::new(
            p_top_right.x - p_top_left.x,
            p_top_right.y - p_top_left.y,
            p_top_right.z - p_top_left.z);
        let qy = Vec3::new(
            p_bottom_left.x - p_top_left.x,
            p_bottom_left.y - p_top_left.y,
            p_bottom_left.z - p_top_left.z);

        debug!("qx:{:?}, qy:{:?}", qx ,qy);

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
            qx: qx,
            qy: qy,
        }
    }
}

impl Ray {
    fn new(ctx: &RayCtx, i: f64, j: f64) -> Ray {
        /* Origin is the eye */
        let x = ctx.p_top_left.x
            + i * ctx.qx.x
            + j * ctx.qy.x
            - ctx.eye.origin.x;
        let y = ctx.p_top_left.y
            + i * ctx.qx.y
            + j * ctx.qy.y
            - ctx.eye.origin.y;
        let z = ctx.p_top_left.z
            + i * ctx.qx.z
            + j * ctx.qy.z
            - ctx.eye.origin.z;

        debug!("({:?}, {:?}) -> ({:?}, {:?}, {:?})", i,j, x, y , z);
        let d = Vec3::new_normalized(x, y, z);
        let r = Ray {origin: ctx.eye.origin.clone(), direction: d};
        r
    }

    pub fn at(&self, t: f64) -> Vec3 {
        self.direction.at(&self.origin, t)
    }
}

fn color(ray: &Ray, scene: &Scene) -> Rgb<u8> {
    let mut distance_min = f64::INFINITY;
    let mut hit_min = Hit::new();

    for o in &scene.objects {
        if let Some(hit) = o.hits(&ray) {
            if hit.t < distance_min {
                distance_min = hit.t;
                hit_min = hit;
            }
        }
    }
    if distance_min == f64::INFINITY {
        let white : Rgb<u8> = Rgb([255, 255, 255]);
        let blue  : Rgb<u8> = Rgb([ 77, 143, 170]);
        let ud = ray.direction.to_normalized();
        scale_rgb(&blue, &white, ud.y).unwrap()
    } else {
        hit_min.to_pixel(distance_min)
    }
}

fn cast_ray_from_eye(ctx: &RayCtx, scene: &Scene, i: f64, j: f64) -> Rgb<u8> {
    let r = Ray::new(&ctx, i, j);
    debug!("({:?},{:?}) r:{:?}", i, j, r);
    assert!(r.direction.z >= 0.);
    color(&r, scene)
}

pub fn render_scene(scene: &Scene, eye: &Eye, nsamples: u64, width: u32, height: u32) -> RgbImage {
    let ctx = RayCtx::new(&eye, width, height);
    let black : Rgb<u8> = Rgb([0, 0, 0]);
    let mut buf: Vec<Rgb<u8>> = vec![black; (width * height) as usize];

    debug!("rendering scene");
    buf.par_iter_mut().enumerate().for_each(
        |(n, pixel)| {
            let y = (n as u32) / width;
            let x = (n as u32) - (y * width);
            let i_min = x as f64 / ctx.width;
            let i_max = (x+1) as f64 / ctx.width;
            let j_min = y as f64 / ctx.height;
            let j_max = (y+1) as f64 / ctx.height;

            let i_step = i_max - i_min;
            let j_step = j_max - j_min;

            let mut r = 0_u64;
            let mut g = 0_u64;
            let mut b = 0_u64;

            let mut rng = rand::thread_rng();

            for _ in 0..nsamples {
                let i = i_min + rng.gen::<f64>() * i_step;
                let j = j_min + rng.gen::<f64>() * j_step;

                let p = cast_ray_from_eye(&ctx, scene, i, j);
                r += p[0] as u64;
                g += p[1] as u64;
                b += p[2] as u64;
            }

            *pixel = Rgb([
                         (r/nsamples) as u8,
                         (g/nsamples) as u8,
                         (b/nsamples) as u8])
        });
    let mut img : RgbImage = ImageBuffer::new(width, height);
    for n in 0..(width*height) {
        let y = (n as u32) / width;
        let x = (n as u32) - (y * width);
        img.put_pixel(x, y, buf[n as usize]);
    };

    img
}
