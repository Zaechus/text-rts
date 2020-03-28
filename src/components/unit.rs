use crate::types::Race;

#[derive(Clone, Debug)]

pub struct Unit {
    race: Race,
    hp: (i32, u32),
    speed: u32,
    damage: u32,
    attack_rate: u32,
    range: u32,
    tic: u32,
}

impl Unit {
    pub fn new(race: Race, hp: i32) -> Self {
        Self {
            race,
            hp: (hp, hp as u32),
            speed: 6,
            damage: 1,
            attack_rate: 50,
            range: 0,
            tic: 0,
        }
    }
    pub fn with_speed(mut self, x: u32) -> Self {
        self.speed = x;
        self
    }
    pub fn with_damage(mut self, x: u32) -> Self {
        self.damage = x;
        self
    }
    pub fn with_range(mut self, r: u32) -> Self {
        self.range = r;
        self
    }

    /// Reduce the Unit's HP by the given value
    pub fn harm(&mut self, x: u32) {
        self.hp.0 -= x as i32;
    }

    pub fn tic(&mut self) {
        if self.tic > 99 {
            self.tic = 0;
        } else {
            self.tic += 1;
        }
    }

    pub fn race(&self) -> Race {
        self.race
    }
    pub fn hp(&self) -> i32 {
        self.hp.0
    }
    pub fn speed(&self) -> u32 {
        self.speed
    }
    pub fn attack(&self) -> Option<u32> {
        if self.tic % self.attack_rate == 0 {
            Some(self.damage)
        } else {
            None
        }
    }
    pub fn range(&self) -> u32 {
        self.range
    }
}
