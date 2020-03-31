use std::time::{SystemTime, UNIX_EPOCH};

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
    pub fn move_towards(&mut self, other: Point) {
        if self.destination.is_none() {
            let a = if self.point.x < other.x {
                1
            } else if self.point.x > other.x {
                -1
            } else {
                0
            };
            let b = if self.point.y < other.y {
                1
            } else if self.point.y > other.y {
                -1
            } else {
                0
            };
            self.destination = Some(Point::new(self.point.x + a, self.point.y + b));
        }
    }
    // pub fn stop_moving(&mut self) {
    //     self.destination = None;
    // }

    pub fn update(&mut self, speed: u32) {
        if self.tic % 51 == 0 {
            self.harmed = false;
        }

        if self.tic > 199 {
            self.tic = 0;
        } else {
            self.tic += 1;
        }

        if let Some(dest) = self.destination {
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

            if self.point == dest {
                self.destination = None;
            }
        }
    }

    /// Given any positive number, move the cell in one of the 4 cardinal directions
    pub fn bump(&mut self, speed: u32) {
        if self.tic % speed == 0 {
            let (a, b) = match SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time oops")
                .as_micros()
                % 8
            {
                0 => (0, -1),
                1 => (1, -1),
                2 => (1, 0),
                3 => (1, 1),
                4 => (0, 1),
                5 => (-1, 1),
                6 => (-1, 0),
                _ => (-1, -1),
            };
            self.point.x += a;
            self.point.y += b;
        }
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

    /// Select the cell
    pub fn select(&mut self) {
        self.selected = true;
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
