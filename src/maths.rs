use rand::Rng;
use image::{
    Rgb,
};
use std::f64;
use std::mem;

pub const EPSILON: f64 = 0.000001;

#[derive(Debug,Clone)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 {
            x: x,
            y: y,
            z: z,
        }
    }

    pub fn origin() -> Vec3 {
        Vec3 { x: 0., y: 0., z: 0., }
    }
    pub fn infinity() -> Vec3 {
        Vec3 { x: f64::INFINITY, y: f64::INFINITY, z: f64::INFINITY, }
    }

    pub fn random_in_unit_sphere() -> Vec3 {
        let mut rng = rand::thread_rng();
        let v = Vec3::new(
            2_f64 * rng.gen::<f64>() - 1_f64,
            2_f64 * rng.gen::<f64>() - 1_f64,
            2_f64 * rng.gen::<f64>() - 1_f64);
        v.to_normalized()
    }
    pub fn new_normalized(x: f64, y: f64, z: f64) -> Vec3 {
        let mut v : Vec3 = Vec3::new(x, y ,z);
        v.normalize();
        v
    }
    pub fn new_clean(&self) -> Vec3 {
        let cleanup = |v| {
            if -EPSILON  <= v && v <= EPSILON {
                0_f64
            } else {
                v
            }
        };
        Vec3 {
            x: cleanup(self.x),
            y: cleanup(self.y),
            z: cleanup(self.z),
        }
    }

    pub fn normalize(&mut self) {
        let d = self.length_sq().sqrt();
        self.x = self.x / d;
        self.y = self.y / d;
        self.z = self.z / d;
    }
    pub fn to_normalized(&self) -> Vec3 {
        let d = self.length_sq().sqrt();
        Vec3 {
            x: self.x / d,
            y: self.y / d,
            z: self.z / d,
        }
    }
    pub fn cross_product(&self, v: &Vec3) -> Vec3 {
        Vec3::new(
            self.y * v.z - self.z * v.y,
            self.z * v.x - self.x * v.z,
            self.x * v.y - self.y * v.x)
    }

    pub fn dot_product(&self, v: &Vec3) -> f64 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    pub fn translate(&self, v: &Vec3, d: f64) -> Vec3 {
        Vec3 {
            x: self.x + v.x * d,
            y: self.y + v.y * d,
            z: self.z + v.z * d,
        }
    }

    pub fn length_sq(&self) -> f64 {
        self.x * self.x +
        self.y * self.y +
        self.z * self.z
    }

    pub fn length_sq_to(&self, p: &Vec3) -> f64 {
        (self.x - p.x) * (self.x - p.x) +
        (self.y - p.y) * (self.y - p.y) +
        (self.z - p.z) * (self.z - p.z)
    }
    pub fn to(&self, dest: &Vec3) -> Vec3 {
        Vec3 {
            x: dest.x - self.x,
            y: dest.y - self.y,
            z: dest.z - self.z,
        }
    }
    pub fn avg(&self, with: &Vec3) -> Vec3 {
        Vec3 {
            x: (with.x + self.x) / 2_f64,
            y: (with.y + self.y) / 2_f64,
            z: (with.z + self.z) / 2_f64,
        }
    }
    pub fn at(&self, from: &Vec3, t: f64) -> Vec3 {
        Vec3 {
            x: from.x + t * self.x,
            y: from.y + t * self.y,
            z: from.z + t * self.z,
        }
    }
    pub fn addv(&self, b: &Vec3) -> Vec3 {
        Vec3 {
            x: self.x + b.x,
            y: self.y + b.y,
            z: self.z + b.z,
        }
    }
    pub fn multv(&self, b: &Vec3) -> Vec3 {
        Vec3 {
            x: self.x * b.x,
            y: self.y * b.y,
            z: self.z * b.z,
        }
    }
    pub fn mult(&mut self, d: f64) {
        self.x *= d;
        self.y *= d;
        self.z *= d;
    }
    pub fn div(&mut self, d: f64) {
        self.x /= d;
        self.y /= d;
        self.z /= d;
    }
    pub fn invert(&mut self) {
        self.x *= -1.;
        self.y *= -1.;
        self.z *= -1.;
    }

    pub fn mixed(&mut self, v: &Vec3, c: f64) {
        self.x = self.x * (1. - c) + v.x * c;
        self.y = self.y * (1. - c) + v.y * c;
        self.z = self.z * (1. - c) + v.z * c;
    }
    pub fn mix(&self, v: &Vec3, c: f64) -> Vec3 {
        Vec3 {
            x: self.x * (1. - c) + v.x * c,
            y: self.y * (1. - c) + v.y * c,
            z: self.z * (1. - c) + v.z * c,
        }
    }
}

pub fn remap_01(a: f64, b: f64, t: f64) -> f64 {
    (t - a) / (b - a)
}

impl Into<Rgb<u8>> for Vec3 {
    fn into(self) -> Rgb<u8> {
        let convert = |v| {
            if v <= 0_f64 {
                return 0_u8;
            }
            if v >= 1_f64 {
                return 255_u8;
            }
            (v * 256_f64) as u8
        };
        let r = convert(self.x);
        let g = convert(self.y);
        let b = convert(self.z);
        Rgb([r, g, b])
    }
}

impl Into<Vec3> for Rgb<u8> {
    fn into(self) -> Vec3 {
        Vec3 {
            x: (self[0] as f64) / 256_f64,
            y: (self[1] as f64) / 256_f64,
            z: (self[2] as f64) / 256_f64,
        }
    }
}
impl Into<Vec3> for &Rgb<u8> {
    fn into(self) -> Vec3 {
        Vec3 {
            x: (self[0] as f64) / 256_f64,
            y: (self[1] as f64) / 256_f64,
            z: (self[2] as f64) / 256_f64,
        }
    }
}



#[derive(Debug,Clone)]
struct Row4 {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
}

pub fn solve_3variable_system(a: &Vec3, b: &Vec3, c: &Vec3, p: &Vec3) -> Option<Vec3> {
    let mut r1 = Row4{a: a.x, b: b.x, c: c.x, d: p.x};
    let mut r2 = Row4{a: a.y, b: b.y, c: c.y, d: p.y};
    let mut r3 = Row4{a: a.z, b: b.z, c: c.z, d: p.z};

    let mut shuffle = Vec3{x: 1., y: 2., z: 3.};
    if r1.a == 0. {
        if r2.a != 0. {
            /* Swap Row1 and Row2 */
            mem::swap(&mut r1, &mut r2);
            mem::swap(&mut shuffle.x, &mut shuffle.y);
        } else if r3.a != 0. {
            /* Swap Row1 and Row3 */
            mem::swap(&mut r1, &mut r3);
            mem::swap(&mut shuffle.x, &mut shuffle.z);
        } else {
            return None;
        }
    }
    if r2.b == 0. {
        if r3.b != 0. {
            /* Swap Row2 and Row3 */
            mem::swap(&mut r2, &mut r3);
            mem::swap(&mut shuffle.y, &mut shuffle.z);
        } else {
            return None;
        }
    }
    if r3.c == 0. {
        return None;
    }

    // 1st step, put a 0 in r3.a using r1 and r3 rows
    if r3.a != 0. {
        let f1 = r1.a;
        let f3 = r3.a;
        r3.a = f1 * r3.a - f3 * r1.a;
        r3.b = f1 * r3.b - f3 * r1.b;
        r3.c = f1 * r3.c - f3 * r1.c;
        r3.d = f1 * r3.d - f3 * r1.d;
        assert!(r3.a == 0.);
    }

    // 2nd step, put a 0 in r2.a using r1 and r2 rows
    if r2.a != 0. {
        let f1 = r1.a;
        let f2 = r2.a;
        r2.a = f1 * r2.a - f2 * r1.a;
        r2.b = f1 * r2.b - f2 * r1.b;
        r2.c = f1 * r2.c - f2 * r1.c;
        r2.d = f1 * r2.d - f2 * r1.d;
        assert!(r2.a == 0.);
    }

    // 3rd step, put a 0 in r2.b using r2 and r3 rows
    if r2.b != 0. {
        let f2 = r2.b;
        let f3 = r3.b;
        r3.a = f2 * r3.a - f3 * r2.a;
        r3.b = f2 * r3.b - f3 * r2.b;
        r3.c = f2 * r3.c - f3 * r2.c;
        r3.d = f2 * r3.d - f3 * r2.d;
        assert!(r3.a == 0.);
    }

    if r3.c == 0. || r2.b == 0. || r1.a == 0. {
        return None
    }

    let mut r = Vec3::origin();
    /* Compute the solution */
    r.z = r3.d / r3.c;
    r.y = (r2.d - r.z * r2.c) / r2.b;
    r.x = (r1.d - r.z * r1.c - r.y * r1.b) / r1.a;

    /* Shuffle back */
    if shuffle.x != 1. {
        if shuffle.y == 1. {
            mem::swap(&mut r.x, &mut r.y);
            mem::swap(&mut shuffle.x, &mut shuffle.y);
        } else {
            mem::swap(&mut r.x, &mut r.z);
            mem::swap(&mut shuffle.x, &mut shuffle.z);
        }
    }
    if shuffle.y != 2. {
            mem::swap(&mut r.y, &mut r.z);
            mem::swap(&mut shuffle.y, &mut shuffle.z);
    }
    assert!(shuffle.x == 1.);
    assert!(shuffle.y == 2.);
    assert!(shuffle.z == 3.);

    return Some(r)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solve_3variable_system_test() {
        let a = Vec3::new(2., 5., -2.);
        let b = Vec3::new(3., -1., -2.);
        let c = Vec3::new(-4., 2., 3.);
        let p = Vec3::new(-5., 15., 3.);
        let r = solve_3variable_system(&a, &b, &c, &p);
        assert!(r.is_some());
        let r = r.unwrap();
        assert_eq!(r.x, 2.);
        assert_eq!(r.y, 1.);
        assert_eq!(r.z, 3.);
    }
}
