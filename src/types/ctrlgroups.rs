use legion::*;

pub struct CtrlGroups {
    groups: Vec<Vec<Entity>>,
    camera: Vec<(i32, i32)>,
}

impl CtrlGroups {
    pub fn new() -> Self {
        Self {
            groups: vec![Vec::new(); 10],
            camera: vec![(0, 0); 3],
        }
    }

    pub fn bind(&mut self, index: usize, units: Vec<Entity>) {
        self.groups[index] = units;
    }

    pub fn add_to(&mut self, index: usize, units: &mut Vec<Entity>) {
        self.groups[index].append(units);
    }

    pub fn group(&self, index: usize) -> Option<&Vec<Entity>> {
        self.groups.get(index)
    }

    pub fn set_cam(&mut self, index: usize, offset: (i32, i32)) {
        self.camera[index] = offset;
    }

    pub fn cam(&self, index: usize) -> (i32, i32) {
        self.camera[index]
    }
}
