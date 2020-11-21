pub struct Map {
    w: i32,
    h: i32,
}

impl Map {
    pub fn new(w: i32, h: i32) -> Self {
        Self { w, h }
    }

    pub fn lower_x(&self) -> i32 {
        -(self.w / 2)
    }
    pub fn upper_x(&self) -> i32 {
        self.w / 2
    }
    pub fn lower_y(&self) -> i32 {
        -(self.h / 2)
    }
    pub fn upper_y(&self) -> i32 {
        self.h / 2
    }
}
