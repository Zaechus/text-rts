use legion::*;

pub struct CtrlGroups {
    groups: Vec<Vec<Entity>>,
}

impl CtrlGroups {
    pub fn new() -> Self {
        Self {
            groups: vec![Vec::new(); 10],
        }
    }

    pub fn bind(&mut self, index: usize, units: Vec<Entity>) {
        self.groups[index] = units;
    }

    pub fn add(&mut self, index: usize, units: &mut Vec<Entity>) {
        self.groups[index].append(units);
    }

    pub fn group(&self, index: usize) -> Option<&Vec<Entity>> {
        self.groups.get(index)
    }
}
