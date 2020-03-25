use bracket_lib::prelude::*;

#[derive(Clone, Debug)]
pub struct GameCell {
    point: Point,
    symbol: char,
    color: RGB,
    selected: bool,
    destination: Option<Point>,
}

impl GameCell {
    pub fn new(x: i32, y: i32, symbol: char, color: RGB) -> Self {
        Self {
            point: Point::new(x, y),
            symbol,
            color,
            selected: false,
            destination: None,
        }
    }

    pub fn move_pos(&mut self, x: i32, y: i32) {
        self.destination = Some(Point::new(x, y));
    }

    pub fn update(&mut self) {
        if let Some(dest) = self.destination {
            if self.point.x <= dest.x - 1 {
                self.point.x += 1;
            } else if self.point.x >= dest.x + 1 {
                self.point.x -= 1;
            }
            if self.point.y <= dest.y - 1 {
                self.point.y += 1;
            } else if self.point.y >= dest.y + 1 {
                self.point.y -= 1;
            }

            if Rect::with_size(dest.x - 1, dest.y - 1, 3, 3).point_in_rect(self.point) {
                self.destination = None;
            }
        }
    }

    pub fn bump(&mut self, n: usize) {
        let (a, b) = match n {
            0 => (-1, 0),
            1 => (1, 0),
            2 => (0, -1),
            _ => (0, 1),
        };
        self.point.x += a;
        self.point.y += b;
    }

    pub fn select(&mut self) {
        self.selected = !self.selected;
    }
    pub fn deselect(&mut self) {
        self.selected = false
    }

    pub fn x(&self) -> i32 {
        self.point.x
    }
    pub fn y(&self) -> i32 {
        self.point.y
    }
    pub fn symbol(&self) -> char {
        self.symbol
    }
    pub fn color(&self) -> RGB {
        self.color
    }
    pub fn color_bright(&self) -> RGB {
        RGB::from_f32(self.color.r * 1.5, self.color.g * 1.5, self.color.b * 1.5)
    }
    pub fn bg_color(&self) -> RGB {
        if self.selected {
            RGB::from_u8(255, 255, 255)
        } else {
            RGB::new()
        }
    }
    pub fn selected(&self) -> bool {
        self.selected
    }
}