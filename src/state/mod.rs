use bracket_lib::prelude::*;

use legion::prelude::*;

use crate::{
    components::{GameCell, Unit},
    types::Race,
};

const GREEN: (u8, u8, u8) = (0, 200, 0);

#[derive(Clone, Debug)]
pub enum CurrentState {
    Menu,
    Playing,
}

enum Mode {
    Select,
    Move,
    Attack,
    Build,
}

pub struct State {
    curr_state: CurrentState,
    world: World,
    window_size: (u32, u32),
    mouse: Point,
    mouse_pressed: bool,
    mouse_released: bool,
    mode: Mode,
    selection: Rect,
}

impl State {
    pub fn new(w: u32, h: u32) -> Self {
        let universe = Universe::new();
        let mut world = universe.create_world();

        let mut units = vec![
            (
                GameCell::new(10, 10, 'V', RGB::named(GREEN)),
                Unit::new(Race::Bionic, 20),
            ),
            (
                GameCell::new(12, 8, 'V', RGB::named(GREEN)),
                Unit::new(Race::Bionic, 20),
            ),
            (
                GameCell::new(14, 12, 'V', RGB::named(GREEN)),
                Unit::new(Race::Bionic, 20),
            ),
        ];
        for x in 0..10 {
            units.push((
                GameCell::new(x + 10, 20 - (x & 1), 'V', RGB::named(GREEN)),
                Unit::new(Race::Bionic, 20),
            ));
        }
        world.insert((), units.into_iter());

        Self {
            curr_state: CurrentState::Menu,
            world,
            window_size: (w, h),
            mouse: Point::new(0, 0),
            mouse_pressed: false,
            mouse_released: false,
            mode: Mode::Select,
            selection: Rect::default(),
        }
    }

    fn menu_state(&mut self, ctx: &mut BTerm) {
        ctx.print_centered(self.window_size.1 as i32 / 2 - 1, "TextRTS");
        ctx.print_centered(
            self.window_size.1 as i32 / 2 + 1,
            "Press the spacebar to start",
        );

        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.curr_state = CurrentState::Playing;
        }
    }

    fn play_state(&mut self, ctx: &mut BTerm) {
        self.mouse = ctx.mouse_point();

        if ctx.left_click {
            if self.mouse_pressed {
                self.mouse_released = true;
            }
            self.mouse_pressed = !self.mouse_pressed;
        }

        self.print_grid(ctx);

        ctx.print(self.mouse.x, self.mouse.y, "<");

        self.print_mode(ctx);

        self.update_cells(ctx);

        self.tmp();

        if self.mouse_released {
            match self.mode {
                Mode::Select => self.select_cells(),
                Mode::Move => {
                    self.move_cells();
                    self.mode = Mode::Select;
                }
                Mode::Attack => (),
                Mode::Build => (),
            }
        }

        self.key_input(ctx);

        self.clear_cells();

        if ctx.left_click {
            self.selection.x1 = self.mouse.x;
            self.selection.y1 = self.mouse.y;
        }
        if self.mouse_pressed {
            self.selection.x2 = self.mouse.x;
            self.selection.y2 = self.mouse.y;
            ctx.draw_hollow_box(
                self.selection.x1,
                self.selection.y1,
                self.selection.width(),
                self.selection.height(),
                RGB::named(GREEN),
                RGB::new(),
            );
        }
        if self.mouse_released {
            self.mouse_released = false;
            self.mouse_pressed = false;
        }
    }

    fn key_input(&mut self, ctx: &mut BTerm) {
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::M => self.mode = Mode::Move,
                VirtualKeyCode::A => self.mode = Mode::Attack,
                VirtualKeyCode::B => self.mode = Mode::Build,
                VirtualKeyCode::Escape => self.mode = Mode::Select,
                _ => (),
            }
        }
    }

    fn print_grid(&mut self, ctx: &mut BTerm) {
        for x in 0..self.window_size.0 {
            for y in 3..self.window_size.1 - 1 {
                ctx.print_color(
                    x as i32,
                    y as i32,
                    RGB::from_u8(100, 100, 100),
                    RGB::new(),
                    ".",
                )
            }
        }
    }

    fn print_mode(&mut self, ctx: &mut BTerm) {
        match self.mode {
            Mode::Select => (),
            Mode::Move => {
                ctx.draw_box(0, 0, 5, 2, RGB::from_u8(0, 175, 0), RGB::from_u8(0, 175, 0));
                ctx.print_color(
                    1,
                    1,
                    RGB::from_u8(255, 255, 255),
                    RGB::from_u8(0, 175, 0),
                    "Move",
                )
            }
            Mode::Attack => {
                ctx.draw_box(0, 0, 7, 2, RGB::from_u8(175, 0, 0), RGB::from_u8(175, 0, 0));
                ctx.print_color(
                    1,
                    1,
                    RGB::from_u8(255, 255, 255),
                    RGB::from_u8(175, 0, 0),
                    "Attack",
                )
            }
            Mode::Build => {
                ctx.draw_box(0, 0, 6, 2, RGB::from_u8(0, 0, 175), RGB::from_u8(0, 0, 175));
                ctx.print_color(
                    1,
                    1,
                    RGB::from_u8(255, 255, 255),
                    RGB::from_u8(0, 0, 175),
                    "Build",
                )
            }
        }
    }

    fn update_cells(&mut self, ctx: &mut BTerm) {
        let query = <(Write<GameCell>,)>::query();

        for (mut cell,) in query.iter(&mut self.world) {
            cell.update();

            ctx.print_color(
                cell.x(),
                cell.y(),
                if self.mouse.x == cell.x() && self.mouse.y == cell.y() {
                    cell.color_bright()
                } else {
                    cell.color()
                },
                cell.bg_color(),
                &cell.symbol().to_string(),
            );
        }
    }

    fn tmp(&mut self) {
        let query = <(Read<GameCell>,)>::query().filter(changed::<Unit>());

        let mut bumped = Vec::new();
        for (e, (cell,)) in query.iter_entities_immutable(&self.world) {
            let query2 = <(Read<GameCell>,)>::query().filter(changed::<Unit>());
            let mut same = false;
            for (e2, (cell2,)) in query2.iter_entities_immutable(&self.world) {
                if e != e2 && cell.x() == cell2.x() && cell.y() == cell2.y() {
                    bumped.push((e2, bumped.len() % 4));
                    same = true;
                    break;
                }
            }
            if same {
                bumped.push((e, bumped.len() % 4));
            }
        }
        for e in bumped {
            if let Some(cell) = self.world.get_component_mut::<GameCell>(e.0).as_deref_mut() {
                cell.bump(e.1);
            }
        }
    }

    fn select_cells(&mut self) {
        let query = <(Write<GameCell>,)>::query();

        for (mut cell,) in query.iter(&mut self.world) {
            if self.selection.width() == 0 || self.selection.height() == 0 {
                if self.mouse.x == cell.x() && self.mouse.y == cell.y() {
                    cell.select();
                    break;
                } else {
                    cell.deselect();
                }
            } else {
                if self.selection.point_in_rect(Point::new(cell.x(), cell.y())) {
                    cell.select();
                } else {
                    cell.deselect();
                }
            }
        }
    }

    fn move_cells(&mut self) {
        let query = <(Write<GameCell>, Write<Unit>)>::query();

        for (mut cell, _) in query.iter(&mut self.world) {
            if cell.selected() {
                cell.move_pos(self.mouse.x, self.mouse.y);
            }
        }
    }

    fn clear_cells(&mut self) {
        let query = <(Read<Unit>,)>::query().filter(changed::<Unit>());

        let mut deleted = Vec::new();
        for (e, (unit,)) in query.iter_entities_immutable(&self.world) {
            if unit.hp() <= 0 {
                deleted.push(e);
            }
        }
        for e in deleted {
            self.world.delete(e);
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        match self.curr_state {
            CurrentState::Menu => self.menu_state(ctx),
            CurrentState::Playing => self.play_state(ctx),
        }
    }
}
