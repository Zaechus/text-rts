use crate::types::Race;

#[derive(Clone, Debug)]

pub struct Unit {
    race: Race,
    hp: (i32, u32),
    speed: f32,
    damage: u32,
    attack_rate: f32,
    range: u32,
    follow_dist: u32,
    tic: f32,
}

impl Unit {
    pub fn new(race: Race, hp: i32) -> Self {
        Self {
            race,
            hp: (hp, hp as u32),
            speed: 13.5,
            damage: 1,
            attack_rate: 1.0,
            range: 0,
            follow_dist: 5,
            tic: 0.0,
        }
    }
    pub fn with_speed(mut self, x: f32) -> Self {
        self.speed = x;
        self
    }
    pub fn with_damage(mut self, x: u32) -> Self {
        self.damage = x;
        self
    }
    pub fn with_range(mut self, r: u32, f: u32) -> Self {
        self.range = r;
        self.follow_dist = f;
        self
    }

    /// Reduce the Unit's HP by the given value
    pub fn harm(&mut self, x: u32) {
        self.hp.0 -= x as i32;
    }

    pub fn tic(&mut self, dt: f32) {
        if self.tic > 5.0 {
            self.tic = 0.0;
        } else {
            self.tic += dt;
        }
    }
    pub fn reset_tic(&mut self) {
        self.tic = 0.0;
    }

    pub fn race(&self) -> Race {
        self.race
    }
    pub fn hp(&self) -> i32 {
        self.hp.0
    }
    pub fn speed(&self) -> f32 {
        self.speed
    }
    pub fn attack(&self) -> Option<u32> {
        if self.tic >= self.attack_rate {
            Some(self.damage)
        } else {
            None
        }
    }
    pub fn range(&self) -> u32 {
        self.range
    }
    pub fn follow_dist(&self) -> u32 {
        self.follow_dist
    }
}
