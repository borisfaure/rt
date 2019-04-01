use crate::object::Object;

#[derive(Debug,Clone)]
pub struct Coords {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}


pub struct Scene {
    pub eye: Coords,
    pub objects: Vec<Box<Object>>,
}

impl Scene {
    pub fn new(eye: Coords) -> Scene {
        Scene {
            eye: eye,
            objects: Vec::new(),
        }
    }
    pub fn add<O: 'static + Object>(&mut self, obj: O) {
        self.objects.push(Box::new(obj));
    }
}
