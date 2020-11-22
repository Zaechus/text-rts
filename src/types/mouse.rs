use bracket_lib::prelude::*;

use crate::types::Mode;

pub struct Mouse {
    pub point: Point,
    pub click: Option<(usize, bool)>,
    pub pressed: (usize, bool, bool),
    cursor: String,
    pub selection: Rect,
}

impl Mouse {
    pub fn new() -> Self {
        Self {
            point: Point::new(0, 0),
            click: None,
            pressed: (0, false, false),
            cursor: String::from("<"),
            selection: Rect::default(),
        }
    }

    pub fn print_cursor(&self, ctx: &mut BTerm, mode: Mode, tic: u8) {
        ctx.print_color(
            self.x(),
            self.y(),
            if let Mode::Attack = mode {
                RGB::named((155 + tic, 0, 0))
            } else {
                RGB::named((0, 155 + tic, 0))
            },
            RGB::new(),
            &self.cursor,
        );
    }

    pub fn select_one(&self) -> bool {
        self.selection.width() == 0 && self.selection.height() == 0
    }
    pub fn point_in_selection(&self, px: i32, py: i32) -> bool {
        let x = if self.selection.x1 <= self.selection.x2 {
            self.selection.x1
        } else {
            self.selection.x2
        };
        let y = if self.selection.y1 <= self.selection.y2 {
            self.selection.y1
        } else {
            self.selection.y2
        };

        Rect::with_size(
            x,
            y,
            self.selection.width() + 1,
            self.selection.height() + 1,
        )
        .point_in_rect(Point::new(px, py))
    }

    pub fn x(&self) -> i32 {
        self.point.x
    }
    pub fn y(&self) -> i32 {
        self.point.y
    }

    pub fn left_pressed(&self) -> bool {
        self.pressed.0 == 0
    }
    pub fn is_pressed(&self) -> bool {
        self.pressed.1
    }
    #[cfg(target_arch = "wasm32")]
    pub fn was_pressed(&self) -> bool {
        self.pressed.2
    }
}
