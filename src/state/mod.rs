#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

use bracket_lib::prelude::*;

use legion::*;

use crate::{
    components::{GameCell, Unit},
    types::{CtrlGroups, Mode, Race, UnitKind},
};

const WHITE: (u8, u8, u8) = (255, 255, 255);
const DARK_GRAY: (u8, u8, u8) = (100, 100, 100);
const BROWN: (u8, u8, u8) = (170, 30, 0);
const GREEN: (u8, u8, u8) = (0, 170, 0);
const DARK_GREEN: (u8, u8, u8) = (0, 120, 0);

#[derive(Clone, Debug)]
pub enum CurrentState {
    Menu,
    Playing,
    Quitting,
}

pub struct State {
    curr_state: CurrentState,
    world: World,
    schedule: Schedule,
    window_size: (u32, u32),
    tic: u8,
    dt: f32,
    #[cfg(not(target_arch = "wasm32"))]
    instant: Instant,
    offset: (i32, i32),
    mouse: Point,
    mouse_click: Option<(usize, bool)>,
    mouse_pressed: (usize, bool, bool),
    mode: Mode,
    cursor: String,
    selection: Rect,
    selected: Vec<Entity>,
    ctrl_groups: CtrlGroups,
}

impl State {
    pub fn new(w: u32, h: u32) -> Self {
        let mut world = World::default();

        let mut units = Vec::new();
        for x in 0..20 {
            units.push((
                GameCell::new(10 - (x & 1), x + 5, 'V', RGB::named(GREEN)),
                Unit::new(Race::Bionic, UnitKind::Blademaster, 30)
                    .with_damage(5)
                    .with_speed(14.5),
            ));
            units.push((
                GameCell::new(7 - (x & 1), x + 5, 'Y', RGB::named(DARK_GREEN)),
                Unit::new(Race::Bionic, UnitKind::Strider, 40)
                    .with_damage(5)
                    .with_range(10, 13),
            ));
        }
        for _ in 0..5 {
            for y in 0..20 {
                units.push((
                    GameCell::new(45, 5 + y, '*', RGB::named(BROWN)),
                    Unit::new(Race::Bug, UnitKind::FleshSpider, 15),
                ));
            }
        }
        world.extend(units);

        let bump_units = SystemBuilder::new("bump_units")
            .with_query(<(Read<GameCell>,)>::query().filter(component::<Unit>()))
            .with_query(<(Read<GameCell>,)>::query())
            .write_component::<GameCell>()
            .build(|_, world, _, (query, inner_query)| {
                let mut bumped = Vec::new();
                for chunk in query.iter_chunks(world) {
                    for (e, (cell,)) in chunk.into_iter_entities() {
                        for inner_chunk in inner_query.iter_chunks(world) {
                            for (e2, (cell2,)) in inner_chunk.into_iter_entities() {
                                if !cell.is_holding() && e != e2 && cell.point() == cell2.point() {
                                    bumped.push(e);
                                    break;
                                }
                            }
                        }
                    }
                }
                for e in bumped.iter() {
                    if let Ok(cell) = world.entry_mut(*e).unwrap().get_component_mut::<GameCell>() {
                        cell.bump();
                    }
                }
            });

        let attack_units = SystemBuilder::new("attack_units")
            .with_query(<(Read<GameCell>, Read<Unit>)>::query())
            .with_query(<(Read<GameCell>, Read<Unit>)>::query())
            .with_query(<(Read<GameCell>, Read<Unit>)>::query())
            .write_component::<GameCell>()
            .write_component::<Unit>()
            .build(|_, world, _, (query, attack_query, moving_query)| {
                let mut attacked_units = Vec::new();
                let mut moving_units = Vec::new();
                for chunk in query.iter_chunks(world) {
                    for (e, (cell, unit)) in chunk.into_iter_entities() {
                        let mut attacked = false;
                        for attack_chunk in attack_query.iter_chunks(world) {
                            for (e2, (cell2, unit2)) in attack_chunk.into_iter_entities() {
                                if e != e2
                                    && unit.race() != unit2.race()
                                    && cell.range_rect(unit.range()).point_in_rect(cell2.point())
                                {
                                    if let Some(damage) = unit.attack() {
                                        attacked_units.push((e, e2, damage));
                                    }
                                    attacked = true;
                                    break;
                                }
                            }
                        }
                        if !attacked {
                            for moving_chunk in moving_query.iter_chunks(world) {
                                for (e2, (cell2, unit2)) in moving_chunk.into_iter_entities() {
                                    if e != e2
                                        && !cell.is_holding()
                                        && unit.race() != unit2.race()
                                        && cell
                                            .range_rect(unit.follow_dist())
                                            .point_in_rect(cell2.point())
                                    {
                                        moving_units.push((e, cell2.point()));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                for (e, e2, dmg) in attacked_units.iter() {
                    if let Ok(cell) = world.entry_mut(*e).unwrap().get_component_mut::<GameCell>() {
                        cell.stop_moving();
                    }
                    if let Ok(unit) = world.entry_mut(*e).unwrap().get_component_mut::<Unit>() {
                        unit.reset_tic();
                    }
                    if let Ok(cell2) = world
                        .entry_mut(*e2)
                        .unwrap()
                        .get_component_mut::<GameCell>()
                    {
                        cell2.set_harmed();
                    }
                    if let Ok(unit2) = world.entry_mut(*e2).unwrap().get_component_mut::<Unit>() {
                        unit2.harm(*dmg);
                    }
                }
                for (e, pt) in moving_units.iter() {
                    if let Ok(cell) = world.entry_mut(*e).unwrap().get_component_mut::<GameCell>() {
                        cell.move_towards(*pt);
                    }
                }
            });

        let clear_units = SystemBuilder::new("clear_units")
            .with_query(<(Read<Unit>,)>::query().filter(maybe_changed::<Unit>()))
            .write_component::<Unit>()
            .build(|commands, world, _, query| {
                let mut deleted = Vec::new();
                for chunk in query.iter_chunks(world) {
                    for (e, (unit,)) in chunk.into_iter_entities() {
                        if unit.hp() <= 0 {
                            deleted.push(e);
                        }
                    }
                }
                for e in deleted.iter() {
                    commands.remove(*e);
                }
            });

        let schedule = Schedule::builder()
            .add_system(bump_units)
            .add_system(attack_units)
            .add_system(clear_units)
            .flush()
            .build();

        Self {
            curr_state: CurrentState::Menu,
            world,
            schedule,
            window_size: (w, h),
            dt: 0.016,
            #[cfg(not(target_arch = "wasm32"))]
            instant: Instant::now(),
            tic: 0,
            offset: (0, 0),
            mouse: Point::new(0, 0),
            mouse_click: None,
            mouse_pressed: (0, false, false),
            mode: Mode::Select,
            cursor: String::from("<"),
            selection: Rect::default(),
            selected: Vec::new(),
            ctrl_groups: CtrlGroups::new(),
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
        let mut resources = Resources::default();
        self.schedule.execute(&mut self.world, &mut resources);

        self.print_grid(ctx);

        ctx.print_color(
            self.mouse.x,
            self.mouse.y,
            RGB::named((0, 155 + self.tic, 0)),
            RGB::new(),
            &self.cursor,
        );

        self.render_cells(ctx);

        self.print_mode(ctx);

        self.mouse_input();

        self.key_input(ctx);

        self.draw_highlight_box(ctx);
    }

    fn mode(&self) -> Mode {
        self.mode
    }
    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    fn mouse_input(&mut self) {
        if self.mouse.x <= 0 {
            self.offset.0 += 1;
        } else if self.mouse.x >= self.window_size.0 as i32 - 1 {
            self.offset.0 -= 1;
        }
        if self.mouse.y <= 0 {
            self.offset.1 += 1;
        } else if self.mouse.y >= self.window_size.1 as i32 - 1 {
            self.offset.1 -= 1;
        }

        match self.mouse_click {
            Some((0, false)) => match self.mode() {
                Mode::Select => self.select_cells(),
                Mode::Move | Mode::Attack => {
                    self.move_cells();
                    self.set_mode(Mode::Select);
                }
                Mode::Ctrl => self.select_same(),
                _ => (),
            },
            Some((1, false)) => {
                self.move_cells();
                self.set_mode(Mode::Select);
            }
            _ => (),
        }
    }

    fn key_num(key: VirtualKeyCode) -> Option<usize> {
        match key {
            VirtualKeyCode::Key0 => Some(0),
            VirtualKeyCode::Key1 => Some(1),
            VirtualKeyCode::Key2 => Some(2),
            VirtualKeyCode::Key3 => Some(3),
            VirtualKeyCode::Key4 => Some(4),
            VirtualKeyCode::Key5 => Some(5),
            VirtualKeyCode::Key6 => Some(6),
            VirtualKeyCode::Key7 => Some(7),
            VirtualKeyCode::Key8 => Some(8),
            VirtualKeyCode::Key9 => Some(9),
            _ => None,
        }
    }

    fn key_input(&mut self, ctx: &mut BTerm) {
        if let Some(key) = ctx.key {
            if let Mode::Ctrl = self.mode {
                if let Some(n) = State::key_num(key) {
                    self.ctrl_groups.bind(n, self.selected.clone());
                }
                self.set_mode(Mode::Select);
            } else if let Mode::Add = self.mode {
                if let Some(n) = State::key_num(key) {
                    self.ctrl_groups.add(n, &mut self.selected.clone());
                }
                self.set_mode(Mode::Select);
            } else {
                match key {
                    VirtualKeyCode::M => self.set_mode(Mode::Move),
                    VirtualKeyCode::A => self.set_mode(Mode::Attack),
                    VirtualKeyCode::B => self.set_mode(Mode::Build),
                    VirtualKeyCode::S => self.stop_cells(),
                    VirtualKeyCode::H => self.hold_cells(),
                    VirtualKeyCode::F => self.focus_cell(),

                    VirtualKeyCode::LControl | VirtualKeyCode::RControl => {
                        self.set_mode(Mode::Ctrl)
                    }
                    VirtualKeyCode::LShift | VirtualKeyCode::RShift => self.set_mode(Mode::Add),

                    VirtualKeyCode::Escape => self.set_mode(Mode::Select),
                    VirtualKeyCode::Up => self.offset.1 += 1,
                    VirtualKeyCode::Down => self.offset.1 -= 1,
                    VirtualKeyCode::Left => self.offset.0 += 1,
                    VirtualKeyCode::Right => self.offset.0 -= 1,
                    VirtualKeyCode::End => self.curr_state = CurrentState::Quitting,
                    _ => {
                        if let Some(n) = State::key_num(key) {
                            if let Some(group) = self.ctrl_groups.group(n) {
                                self.selected = group.clone();
                                self.load_ctrl_group();
                            }
                        }
                    }
                }
            }
        }
    }

    fn draw_highlight_box(&mut self, ctx: &mut BTerm) {
        if let Mode::Select = self.mode {
            self.selection.x2 = self.mouse.x;
            self.selection.y2 = self.mouse.y;
            if self.mouse_pressed.0 == 0 && self.mouse_pressed.1 {
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
            } else {
                self.selection.x1 = self.mouse.x;
                self.selection.y1 = self.mouse.y;
            }
        }
    }

    fn print_grid(&mut self, ctx: &mut BTerm) {
        for x in 0..self.window_size.0 {
            for y in 0..self.window_size.1 - 1 {
                ctx.print_color(x as i32, y as i32, RGB::named(DARK_GRAY), RGB::new(), ".")
            }
        }
    }

    fn print_mode(&mut self, ctx: &mut BTerm) {
        let mut w = 0;
        let mut color = RGB::new();
        let mut s = "";
        match self.mode() {
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
            Mode::Ctrl => {
                w = 5;
                color = RGB::from_u8(75, 75, 75);
                s = "Ctrl"
            }
            Mode::Add => {
                w = 4;
                color = RGB::from_u8(75, 75, 75);
                s = "Add"
            }
            _ => (),
        }
        ctx.draw_box(0, 0, w, 2, color, color);
        ctx.print_color(1, 1, RGB::named(WHITE), color, s)
    }

    fn render_cells(&mut self, ctx: &mut BTerm) {
        let mut query = <(Write<GameCell>, Write<Unit>)>::query();

        for (cell, unit) in query.iter_mut(&mut self.world) {
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

            cell.update(self.dt, unit.speed());
            unit.tic(self.dt);
        }
    }

    fn load_ctrl_group(&mut self) {
        let mut query = <(Write<GameCell>,)>::query();

        for chunk in query.iter_chunks_mut(&mut self.world) {
            for (e, (cell,)) in chunk.into_iter_entities() {
                if self.selected.contains(&e) {
                    cell.select();
                } else {
                    cell.deselect();
                }
            }
        }
    }

    fn select_cells(&mut self) {
        let mut query = <(Write<GameCell>,)>::query();

        self.selected = Vec::new();

        if self.selection.width() == 0 || self.selection.height() == 0 {
            for chunk in query.iter_chunks_mut(&mut self.world) {
                for (e, (cell,)) in chunk.into_iter_entities() {
                    if self.mouse.x == cell.x() + self.offset.0
                        && self.mouse.y == cell.y() + self.offset.1
                    {
                        cell.select();
                        self.selected.push(e);
                        break;
                    } else {
                        cell.deselect();
                    }
                }
            }
        } else {
            for chunk in query.iter_chunks_mut(&mut self.world) {
                for (e, (cell,)) in chunk.into_iter_entities() {
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
                        self.selected.push(e);
                    } else {
                        cell.deselect();
                    }
                }
            }
        }
    }

    fn select_same(&mut self) {
        let mut query = <(Write<GameCell>, Read<Unit>)>::query();

        self.selected = Vec::new();

        let mut kind = None;

        if self.selection.width() == 0 || self.selection.height() == 0 {
            for (cell, unit) in query.iter_mut(&mut self.world) {
                if self.mouse.x == cell.x() + self.offset.0
                    && self.mouse.y == cell.y() + self.offset.1
                {
                    kind = Some(unit.kind());
                    break;
                }
            }
        }
        if let Some(kind) = kind {
            for chunk in query.iter_chunks_mut(&mut self.world) {
                for (e, (cell, unit)) in chunk.into_iter_entities() {
                    if kind == unit.kind()
                        && cell.x() + self.offset.0 > 0
                        && cell.y() + self.offset.1 > 0
                        && cell.x() + self.offset.0 < self.window_size.0 as i32
                        && cell.y() + self.offset.1 < self.window_size.1 as i32
                    {
                        cell.select();
                        self.selected.push(e);
                    } else {
                        cell.deselect();
                    }
                }
            }
        }

        self.mode = Mode::Select;
    }

    fn move_cells(&mut self) {
        let mut query = <(Write<GameCell>, Write<Unit>)>::query();

        for (cell, unit) in query.iter_mut(&mut self.world) {
            if cell.selected() {
                cell.move_pos(Point::new(
                    self.mouse.x - self.offset.0,
                    self.mouse.y - self.offset.1,
                ));
                unit.reset_tic();
            }
        }
    }

    fn stop_cells(&mut self) {
        let mut query = <(Write<GameCell>, Write<Unit>)>::query();

        for (cell, _) in query.iter_mut(&mut self.world) {
            if cell.selected() {
                cell.stop_moving();
            }
        }
    }

    fn hold_cells(&mut self) {
        let mut query = <(Write<GameCell>, Write<Unit>)>::query();

        for (cell, _) in query.iter_mut(&mut self.world) {
            if cell.selected() {
                cell.stop_moving();
                cell.hold();
            }
        }
    }

    fn focus_cell(&mut self) {
        let mut query = <(Write<GameCell>, Write<Unit>)>::query();

        for (cell, _) in query.iter_mut(&mut self.world) {
            if cell.selected() {
                self.offset = (
                    -cell.x() + self.window_size.0 as i32 / 2,
                    -cell.y() + self.window_size.1 as i32 / 2,
                );
            }
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

    #[cfg(target_arch = "wasm32")]
    fn update_dt(&self) {}
    #[cfg(not(target_arch = "wasm32"))]
    fn update_dt(&mut self) {
        self.dt = Instant::now().duration_since(self.instant).as_secs_f32();
        self.instant = Instant::now();
    }

    #[cfg(target_arch = "wasm32")]
    fn get_input(&mut self) {
        self.mouse_pressed.2 = false;

        let mut input = INPUT.lock();

        input.for_each_message(|event| match event {
            BEvent::MouseButtonUp { button } => {
                self.mouse_pressed = (button, false, self.mouse_pressed.1)
            }
            BEvent::MouseButtonDown { button } => {
                self.mouse_pressed = (button, true, self.mouse_pressed.1)
            }
            _ => (),
        });

        if !self.mouse_pressed.1 && self.mouse_pressed.2 {
            self.mouse_click = Some((self.mouse_pressed.0, false))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn get_input(&mut self) {
        let mut input = INPUT.lock();

        input.for_each_message(|event| match event {
            BEvent::MouseClick { button, pressed } => self.mouse_click = Some((button, pressed)),
            BEvent::MouseButtonUp { button } => self.mouse_pressed = (button, false, false),
            BEvent::MouseButtonDown { button } => self.mouse_pressed = (button, true, false),
            _ => (),
        });
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        self.update_dt();

        ctx.cls();

        self.get_input();

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
