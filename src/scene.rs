use crate::object::Object;
use crate::maths::{
    Vec3,
};


pub struct Scene {
    pub objects: Vec<Box<Object + Sync + Send>>,
    pub sun: Option<(Vec3, Vec3, f64)>,
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            objects: Vec::new(),
            sun: None,
        }
    }
    pub fn add<O: 'static + Object + Sync + Send>(&mut self, obj: O) {
        self.objects.push(Box::new(obj));
    }
    pub fn set_sun(&mut self, sun: Option<(Vec3, Vec3, f64)>) {
        self.sun = sun;
    }
    pub fn set_golden_sun(&mut self) {
        self.set_sun(
            Some(
                (Vec3::new(3., 1., 0.),
                Vec3::new(1.,1.,1.),
                0.6)));
    }
}
