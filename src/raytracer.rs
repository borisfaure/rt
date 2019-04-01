use image::{
    Rgb,
    RgbImage,
};
use crate::scene::{
    Coords,
    Scene,
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

pub struct Ray {
    pub o: Coords,
    pub d: Coords,
}

impl Ray {
    fn new(scene: &Scene, x: f64, y: f64) -> Ray {
        let o = Coords {x: 0., y: 0., z: 0.,};
        let d = Coords {x: 0., y: 0., z: 0.,};
        let r = Ray {o: o, d: d};
        r
    }
}

fn cast_ray(scene: &Scene, x: f64, y: f64) -> Rgb<u8> {
    let r = Ray::new(scene, x, y);
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

pub fn render_scene(scene: &Scene, img: &mut RgbImage) {
    // Obtain the image's width and height.
    let (width, height) = img.dimensions();
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        *pixel = cast_ray(scene,
                          x as f64 / width as f64,
                          y as f64 / height as f64);
    }
}
