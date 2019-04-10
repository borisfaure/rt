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
    pub fn new_normalized(x: f64, y: f64, z: f64) -> Vec3 {
        let mut v : Vec3 = Vec3::new(x, y ,z);
        v.normalize();
        v
    }
    pub fn normalize(&mut self) {
        let d = (self.x * self.x +
                 self.y * self.y +
                 self.z * self.z).sqrt();
        self.x = self.x / d;
        self.y = self.y / d;
        self.z = self.z / d;
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
    pub fn length_sq(&self, p: &Vec3) -> f64 {
        (self.x - p.x) * (self.x - p.x) +
        (self.y - p.y) * (self.y - p.y) +
        (self.z - p.z) * (self.z - p.z)
    }
}

pub fn remap_01(a: f64, b: f64, t: f64) -> f64 {
    (t - a) / (b - a)
}
