use image::{
    Rgb,
    RgbImage,
};
use crate::scene::{
    Scene,
};
use crate::maths::{
    Coords,
    Vector,
};
use std::f64;

pub struct Shading {
    pub color: Rgb<u8>,
}

impl Shading {
    fn new() -> Shading {
        Shading {
            color: Rgb([0, 0, 0]),
        }
    }
    fn to_pixel(&self, _distance: f64) -> Rgb<u8> {
        /* TODO: */
        self.color
    }
}

#[derive(Debug,Clone)]
pub struct Eye {
    pub origin: Coords,
    pub direction: Vector,
}

#[derive(Debug,Clone)]
pub struct Ray {
    pub o: Coords,
    pub d: Vector,
}

struct RayCtx {
    eye: Eye,
    w: Vector,
    b: Vector,
    v: Vector,
    c: Coords,
    p_top_left: Coords,
    p_top_right: Coords,
    p_bottom_left: Coords,
    width: f64,
    height: f64,
    hx: f64,
    hy: f64,
    qx: Vector,
    qy: Vector,
}
impl RayCtx {
    fn new(eye: &Eye, img: &RgbImage) -> RayCtx {
        let w = Vector::new(0., 1., 0.);
        let b = w.cross_product(&eye.direction);
        let v = eye.direction.cross_product(&b);
        debug!("b:{:?} v:{:?}", b, v);
        let d = 1.;
        let c = eye.origin.translate(&eye.direction, d);
        // Obtain the image's width and height.
        let (width, height) = img.dimensions();
        let width = width as f64;
        let height = height as f64;
        let aspect_ratio = 1_f64;

        /* Use 90° as horizontal field of view
         * Tan(π/4) = 1
         */
        let hx = d;
        let hy = hx / aspect_ratio;

        let p_top_left = Coords::new(
            c.x - 0.5 * b.x + 0.5 * v.x,
            c.y - 0.5 * b.y + 0.5 * v.y,
            c.z - 0.5 * b.z + 0.5 * v.z);
        let p_top_right = Coords::new(
            c.x + 0.5 * b.x + 0.5 * v.x,
            c.y + 0.5 * b.y + 0.5 * v.y,
            c.z + 0.5 * b.z + 0.5 * v.z);
        let p_bottom_left = Coords::new(
            c.x - 0.5 * b.x - 0.5 * v.x,
            c.y - 0.5 * b.y - 0.5 * v.y,
            c.z - 0.5 * b.z - 0.5 * v.z);
        debug!("top_left:{:?} top_right:{:?} bottom_left:{:?}",
               p_top_left, p_top_right, p_bottom_left);
        let qx = Vector::new(
            p_top_right.x - p_top_left.x,
            p_top_right.y - p_top_left.y,
            p_top_right.z - p_top_left.z);
        let qy = Vector::new(
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
        let d = Vector::new_normalized(x, y, z);
        let r = Ray {o: ctx.eye.origin.clone(), d: d};
        r
    }
}

fn cast_ray(ctx: &RayCtx, scene: &Scene, x: u32, y: u32) -> Rgb<u8> {
    let i = x as f64 / ctx.width;
    let j = y as f64 / ctx.height;
    let r = Ray::new(&ctx, i, j);
    debug!("({:?},{:?}) r:{:?}", i, j, r);
    let mut distance_min = f64::INFINITY;
    let mut sh_min = Shading::new();

    for o in &scene.objects {
        if let Some((distance, shading)) = o.intersects(&r) {
            if distance < distance_min {
                distance_min = distance;
                sh_min = shading;
            }
        }
    }
    sh_min.to_pixel(distance_min)
}

pub fn render_scene(scene: &Scene, eye: &Eye, img: &mut RgbImage) {
    let ctx = RayCtx::new(&eye, &img);

    debug!("rendering scene");
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        *pixel = cast_ray(&ctx, scene, x, y);
    }
}
