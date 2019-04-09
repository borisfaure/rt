
#[derive(Debug,Clone)]
pub struct Coords {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
impl Coords {
    pub fn new(x: f64, y: f64, z: f64) -> Coords {
        Coords {
            x: x,
            y: y,
            z: z,
        }
    }
    pub fn translate(&self, v: &Vector, d: f64) -> Coords {
        Coords {
            x: self.x + v.x * d,
            y: self.y + v.y * d,
            z: self.z + v.z * d,
        }
    }
    pub fn length_sq(&self, p: &Coords) -> f64 {
        (self.x - p.x) * (self.x - p.x) +
        (self.y - p.y) * (self.y - p.y) +
        (self.z - p.z) * (self.z - p.z)
    }
}

#[derive(Debug,Clone)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector {
    pub fn new(x: f64, y: f64, z: f64) -> Vector {
        Vector {
            x: x,
            y: y,
            z: z,
        }
    }
    pub fn new_normalized(x: f64, y: f64, z: f64) -> Vector {
        let mut v : Vector = Vector::new(x, y ,z);
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
    pub fn cross_product(&self, v: &Vector) -> Vector {
        Vector::new(
            self.y * v.z - self.z * v.y,
            self.z * v.x - self.x * v.z,
            self.x * v.y - self.y * v.x)
    }
    pub fn dot_product(&self, v: &Vector) -> f64 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }
}


#[derive(Debug,Clone)]
pub struct Plane {
    pub center: Coords,
    pub normal: Vector,
}
