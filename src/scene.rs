use crate::object::Object;

#[derive(Debug,Clone)]
pub struct Coord {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}


pub struct Scene {
    camera: Coord,
    objects: Vec<Box<Object>>,
}

impl Scene {
    pub fn new(camera: Coord) -> Scene {
        Scene {
            camera: camera,
            objects: Vec::new(),
        }
    }
}
