use bracket_lib::prelude::*;

use legion::prelude::*;

use crate::{
    components::{GameCell, Unit},
    types::Race,
};

const BROWN: (u8, u8, u8) = (170, 20, 0);
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
    tic: u8,
    offset: (i32, i32),
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

        let mut units = Vec::new();
        for x in 0..10 {
            units.push((
                GameCell::new(x + 10, 20 - (x & 1), 'V', RGB::named(GREEN)),
                Unit::new(Race::Bionic, 30).with_damage(5).with_speed(4),
            ));
        }
        for x in 0..5 {
            for y in 0..30 {
                units.push((
                    GameCell::new(35 - (x & 1), 5 + y, '*', RGB::named(BROWN)),
                    Unit::new(Race::Bug, 15),
                ));
            }
        }
        world.insert((), units.into_iter());

        Self {
            curr_state: CurrentState::Menu,
            world,
            window_size: (w, h),
            tic: 0,
            offset: (0, 0),
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
        self.tic += 4;
        if self.tic > 99 {
            self.tic = 0;
        }

        self.mouse = ctx.mouse_point();

        if ctx.left_click {
            if self.mouse_pressed {
                self.mouse_released = true;
            }
            self.mouse_pressed = !self.mouse_pressed;
        }

        self.print_grid(ctx);

        ctx.print_color(
            self.mouse.x,
            self.mouse.y,
            RGB::named((0, 155 + self.tic, 0)),
            RGB::new(),
            "<",
        );

        self.print_mode(ctx);

        self.update_cells(ctx);

        self.bump_cells();

        self.attack_units();

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
        self.mouse_released = false;
    }

    fn key_input(&mut self, ctx: &mut BTerm) {
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::M => self.mode = Mode::Move,
                VirtualKeyCode::A => self.mode = Mode::Attack,
                VirtualKeyCode::B => self.mode = Mode::Build,
                VirtualKeyCode::Escape => self.mode = Mode::Select,
                VirtualKeyCode::Up => self.offset.1 += 1,
                VirtualKeyCode::Down => self.offset.1 -= 1,
                VirtualKeyCode::Left => self.offset.0 += 1,
                VirtualKeyCode::Right => self.offset.0 -= 1,
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
        let mut w = 0;
        let mut color = RGB::new();
        let mut s = "";
        match self.mode {
            Mode::Select => (),
            Mode::Move => {
                w = 5;
                color = RGB::from_u8(0, 175, 0);
                s = "Move";
            }
            Mode::Attack => {
                w = 7;
                color = RGB::from_u8(175, 0, 0);
                s = "Attack";
            }
            Mode::Build => {
                w = 6;
                color = RGB::from_u8(0, 0, 175);
                s = "Build";
            }
        }
        ctx.draw_box(0, 0, w, 2, color, color);
        ctx.print_color(1, 1, RGB::from_u8(255, 255, 255), color, s)
    }

    fn update_cells(&mut self, ctx: &mut BTerm) {
        let query = <(Write<GameCell>, Write<Unit>)>::query();

        for (mut cell, mut unit) in query.iter(&mut self.world) {
            ctx.print_color(
                cell.x() + self.offset.0,
                cell.y() + self.offset.1,
                if self.mouse.x - self.offset.0 == cell.x()
                    && self.mouse.y - self.offset.1 == cell.y()
                {
                    cell.color_bright()
                } else {
                    cell.color()
                },
                cell.bg_color(),
                &cell.symbol().to_string(),
            );

            cell.update(unit.speed());
            unit.tic();
        }
    }

    fn bump_cells(&mut self) {
        let query = <(Read<GameCell>,)>::query();

        let mut bumped = Vec::new();
        for (e, (cell,)) in query.iter_entities_immutable(&self.world) {
            let query2 = <(Read<GameCell>,)>::query();
            let mut same = false;
            for (e2, (cell2,)) in query2.iter_entities_immutable(&self.world) {
                if e != e2 && cell.x() == cell2.x() && cell.y() == cell2.y() {
                    bumped.push((e2, bumped.len()));
                    same = true;
                    break;
                }
            }
            if same {
                bumped.push((e, bumped.len()));
            }
        }
        for (e, dir) in bumped {
            if let Some(cell) = self.world.get_component_mut::<GameCell>(e).as_deref_mut() {
                cell.bump(dir);
            }
        }
    }

    fn select_cells(&mut self) {
        let query = <(Write<GameCell>,)>::query();

        for (mut cell,) in query.iter(&mut self.world) {
            if self.selection.width() == 0 || self.selection.height() == 0 {
                if self.mouse.x == cell.x() + self.offset.0
                    && self.mouse.y == cell.y() + self.offset.1
                {
                    cell.select();
                    break;
                } else {
                    cell.deselect();
                }
            } else if self.selection.point_in_rect(Point::new(
                cell.x() + self.offset.0,
                cell.y() + self.offset.1,
            )) {
                cell.select();
            } else {
                cell.deselect();
            }
        }
    }

    fn move_cells(&mut self) {
        let query = <(Write<GameCell>, Write<Unit>)>::query();

        for (mut cell, _) in query.iter(&mut self.world) {
            if cell.selected() {
                cell.move_pos(Point::new(
                    self.mouse.x - self.offset.0,
                    self.mouse.y - self.offset.1,
                ));
            }
        }
    }

    fn attack_units(&mut self) {
        let query = <(Read<GameCell>, Read<Unit>)>::query();

        let mut attacked = Vec::new();
        for (e, (cell, unit)) in query.iter_entities_immutable(&self.world) {
            let query2 = <(Read<GameCell>, Read<Unit>)>::query();
            for (e2, (cell2, unit2)) in query2.iter_entities_immutable(&self.world) {
                if e != e2
                    && unit.race() != unit2.race()
                    && cell.range_rect(unit.range()).point_in_rect(cell2.point())
                {
                    if let Some(damage) = unit.attack() {
                        attacked.push((e2, damage));
                        break;
                    }
                }
            }
        }

        for (e, dmg) in attacked {
            if let Some(cell) = self.world.get_component_mut::<GameCell>(e).as_deref_mut() {
                cell.set_harmed();
            }
            if let Some(unit) = self.world.get_component_mut::<Unit>(e).as_deref_mut() {
                unit.harm(dmg);
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
