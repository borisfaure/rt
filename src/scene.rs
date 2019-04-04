use crate::object::Object;


pub struct Scene {
    pub objects: Vec<Box<Object>>,
}

impl Scene {
    pub fn new() -> Scene {
        Scene {
            objects: Vec::new(),
        }
    }
    pub fn add<O: 'static + Object>(&mut self, obj: O) {
        self.objects.push(Box::new(obj));
    }
}
