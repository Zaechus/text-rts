use std::sync::{Arc, Mutex};

use bracket_lib::prelude::*;

use legion::prelude::*;

use crate::{
    components::{GameCell, Unit},
    types::Race,
};

const BROWN: (u8, u8, u8) = (170, 30, 0);
const GREEN: (u8, u8, u8) = (0, 170, 0);

#[derive(Clone, Debug)]
pub enum CurrentState {
    Menu,
    Playing,
    Quitting,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Mode {
    Select,
    Move,
    Attack,
    Build,
}

add_wasm_support!();

pub struct State {
    curr_state: CurrentState,
    world: World,
    window_size: (u32, u32),
    tic: u8,
    offset: (i32, i32),
    mouse: Point,
    mouse_click: Option<(usize, bool)>,
    mouse_pressed: (usize, bool),
    mode: Mode,
    cursor: String,
    selection: Rect,
}

impl State {
    pub fn new(w: u32, h: u32) -> Self {
        let universe = Universe::new();
        let mut world = universe.create_world();

        let mut units = Vec::new();
        for x in 0..20 {
            units.push((
                GameCell::new(10 - (x & 1), x + 5, 'V', RGB::named(GREEN)),
                Unit::new(Race::Bionic, 30).with_damage(5).with_speed(5),
            ));
            units.push((
                GameCell::new(7 - (x & 1), x + 5, 'Y', RGB::named(GREEN)),
                Unit::new(Race::Bionic, 40).with_damage(5).with_range(10),
            ));
        }
        for _ in 0..5 {
            for y in 0..20 {
                units.push((
                    GameCell::new(45, 5 + y, '*', RGB::named(BROWN)),
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
            mouse_click: None,
            mouse_pressed: (0, false),
            mode: Mode::Select,
            cursor: String::from("<"),
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
        self.print_grid(ctx);

        ctx.print_color(
            self.mouse.x,
            self.mouse.y,
            RGB::named((0, 155 + self.tic, 0)),
            RGB::new(),
            &self.cursor,
        );

        self.update_cells(ctx);

        self.print_mode(ctx);

        self.bump_attack_clear_units();

        match self.mouse_click {
            Some((0, false)) => match self.mode {
                Mode::Select => self.select_cells(),
                Mode::Move | Mode::Attack => {
                    self.move_cells();
                    self.mode = Mode::Select;
                }
                Mode::Build => (),
            },
            Some((1, false)) => {
                self.move_cells();
                self.mode = Mode::Select;
            }
            Some((0, true)) => {
                self.selection.x1 = self.mouse.x;
                self.selection.y1 = self.mouse.y;
            }
            _ => (),
        }

        self.key_input(ctx);

        self.draw_highlight_box(ctx);
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
                VirtualKeyCode::End => self.curr_state = CurrentState::Quitting,
                _ => (),
            }
        }
    }

    fn draw_highlight_box(&mut self, ctx: &mut BTerm) {
        if self.mouse_pressed == (0, true) {
            self.selection.x2 = self.mouse.x;
            self.selection.y2 = self.mouse.y;
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
            ctx.draw_hollow_box(
                x,
                y,
                self.selection.width(),
                self.selection.height(),
                RGB::named(GREEN),
                RGB::new(),
            );
        }
    }

    fn print_grid(&mut self, ctx: &mut BTerm) {
        for x in 0..self.window_size.0 {
            for y in 0..self.window_size.1 - 1 {
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
            _ => (),
        }
        ctx.draw_box(0, 0, w, 2, color, color);
        ctx.print_color(1, 1, RGB::from_u8(255, 255, 255), color, s)
    }

    fn update_cells(&mut self, ctx: &mut BTerm) {
        let query = <(Write<GameCell>, Write<Unit>)>::query();

        for (mut cell, mut unit) in query.iter(&mut self.world) {
            if Rect::with_exact(
                -self.offset.0,
                -self.offset.1,
                self.window_size.0 as i32 - self.offset.0,
                self.window_size.1 as i32 - self.offset.1,
            )
            .point_in_rect(cell.point())
            {
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
            }

            cell.update(unit.speed());
            unit.tic();
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
            } else {
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
                if Rect::with_size(
                    x,
                    y,
                    self.selection.width() + 1,
                    self.selection.height() + 1,
                )
                .point_in_rect(Point::new(
                    cell.x() + self.offset.0,
                    cell.y() + self.offset.1,
                )) {
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
                cell.move_pos(Point::new(
                    self.mouse.x - self.offset.0,
                    self.mouse.y - self.offset.1,
                ));
            }
        }
    }

    fn bump_attack_clear_units(&mut self) {
        let query = <(Read<GameCell>, Read<Unit>)>::query();

        let bumped = Arc::new(Mutex::new(Vec::new()));
        let attacked_units = Arc::new(Mutex::new(Vec::new()));
        let moving_units = Arc::new(Mutex::new(Vec::new()));
        let deleted = Arc::new(Mutex::new(Vec::new()));
        query.par_entities_for_each_immutable(&self.world, |(e, (cell, unit))| {
            let query2 = <(Read<GameCell>,)>::query();
            for (e2, (cell2,)) in query2.iter_entities_immutable(&self.world) {
                if e != e2 && cell.point() == cell2.point() {
                    bumped.lock().unwrap().push((e, unit.speed()));
                    break;
                }
            }
            let query2 = <(Read<GameCell>, Read<Unit>)>::query();
            let mut attacked = false;
            for (e2, (cell2, unit2)) in query2.iter_entities_immutable(&self.world) {
                if e != e2
                    && unit.race() != unit2.race()
                    && cell.range_rect(unit.range()).point_in_rect(cell2.point())
                {
                    if let Some(damage) = unit.attack() {
                        attacked_units.lock().unwrap().push((e2, damage));
                    }
                    attacked = true;
                    break;
                }
            }
            if !attacked {
                let query2 = <(Read<GameCell>, Read<Unit>)>::query();
                for (e2, (cell2, unit2)) in query2.iter_entities_immutable(&self.world) {
                    if e != e2
                        && unit.race() != unit2.race()
                        && cell
                            .range_rect((unit.range() as f32 * 0.5 + 6.0).floor() as u32)
                            .point_in_rect(cell2.point())
                    {
                        moving_units.lock().unwrap().push((e, cell2.point()));
                        break;
                    }
                }
            }
            if unit.hp() <= 0 {
                deleted.lock().unwrap().push(e);
            }
        });

        for (e, speed) in bumped.lock().unwrap().iter() {
            if let Some(cell) = self.world.get_component_mut::<GameCell>(*e).as_deref_mut() {
                cell.bump(*speed);
            }
        }
        for (e, dmg) in attacked_units.lock().unwrap().iter() {
            if let Some(cell) = self.world.get_component_mut::<GameCell>(*e).as_deref_mut() {
                cell.set_harmed();
            }
            if let Some(unit) = self.world.get_component_mut::<Unit>(*e).as_deref_mut() {
                unit.harm(*dmg);
            }
        }
        for (e, pt) in moving_units.lock().unwrap().iter() {
            if let Some(cell) = self.world.get_component_mut::<GameCell>(*e).as_deref_mut() {
                cell.move_towards(*pt);
            }
        }
        for e in deleted.lock().unwrap().iter() {
            self.world.delete(*e);
        }
    }

    fn quit_state(&mut self, ctx: &mut BTerm) {
        ctx.print(5, 5, "Are you sure you want to quit? (y/n)");

        if let Some(VirtualKeyCode::Y) = ctx.key {
            ctx.quit();
        } else if let Some(VirtualKeyCode::N) = ctx.key {
            self.curr_state = CurrentState::Playing;
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        let mut input = INPUT.lock();

        input.for_each_message(|event| match event {
            BEvent::MouseClick { button, pressed } => self.mouse_click = Some((button, pressed)),
            BEvent::MouseButtonUp { button } => self.mouse_pressed = (button, false),
            BEvent::MouseButtonDown { button } => self.mouse_pressed = (button, true),
            _ => (),
        });

        self.tic += 4;
        if self.tic > 99 {
            self.tic = 0;
        }

        self.mouse = ctx.mouse_point();

        match self.curr_state {
            CurrentState::Menu => self.menu_state(ctx),
            CurrentState::Playing => self.play_state(ctx),
            CurrentState::Quitting => self.quit_state(ctx),
        }

        self.mouse_click = None;
    }
}
