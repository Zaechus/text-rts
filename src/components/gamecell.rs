use bracket_lib::prelude::*;

#[derive(Clone, Debug)]
pub struct GameCell {
    point: Point,
    symbol: char,
    color: RGB,
    selected: bool,
    destination: Option<Point>,
    tic: u32,
    harmed: bool,
}

impl GameCell {
    pub fn new(x: i32, y: i32, symbol: char, color: RGB) -> Self {
        Self {
            point: Point::new(x, y),
            symbol,
            color,
            selected: false,
            destination: None,
            tic: 0,
            harmed: false,
        }
    }

    pub fn move_pos(&mut self, point: Point) {
        self.destination = Some(point);
    }

    pub fn update(&mut self, speed: u32) {
        self.harmed = false;

        if let Some(dest) = self.destination {
            self.tic += 1;

            if self.tic % speed == 0 {
                if self.point.x < dest.x {
                    self.point.x += 1;
                } else if self.point.x > dest.x {
                    self.point.x -= 1;
                }
                if self.point.y < dest.y {
                    self.point.y += 1;
                } else if self.point.y > dest.y {
                    self.point.y -= 1;
                }
            }

            if Rect::with_size(dest.x - 1, dest.y - 1, 2, 2).point_in_rect(self.point) {
                self.destination = None;
                self.tic = 0;
            }
        }
    }

    /// Given any positive number, move the cell in one of the 4 cardinal directions
    pub fn bump(&mut self, n: usize) {
        let (a, b) = match n % 4 {
            0 => (-1, 0),
            1 => (0, 1),
            2 => (1, 0),
            _ => (0, -1),
        };
        self.point.x += a;
        self.point.y += b;
    }

    /// Given a positive range value, return the range of the cell as a Rect
    pub fn range_rect(&self, r: u32) -> Rect {
        let r = r as i32;
        Rect::with_size(
            self.point.x - r - 1,
            self.point.y - r - 1,
            r * 2 + 2,
            r * 2 + 2,
        )
    }

    /// Toggle the selected status of the cell
    pub fn select(&mut self) {
        self.selected = !self.selected;
    }
    /// Deselect the cell
    pub fn deselect(&mut self) {
        self.selected = false
    }

    /// Set the harmed status of the cell to true, causing it to appear red
    pub fn set_harmed(&mut self) {
        self.harmed = true;
    }

    pub fn point(&self) -> Point {
        self.point
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
    /// Return the RGB color of the cell; if harmed return red
    pub fn color(&self) -> RGB {
        if self.harmed {
            RGB::named((255, 0, 0))
        } else {
            self.color
        }
    }
    /// Return a brightened version of the cell's color
    pub fn color_bright(&self) -> RGB {
        RGB::from_f32(self.color.r * 1.5, self.color.g * 1.5, self.color.b * 1.5)
    }
    /// Return a black background for the cell, but black if selected
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
