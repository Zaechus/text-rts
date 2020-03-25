use crate::types::Race;

#[derive(Clone, Debug)]

pub struct Unit {
    race: Race,
    hp: (i32, u32),
    speed: u32,
    damage: u32,
    range: u32,
}

impl Unit {
    pub fn new(race: Race, hp: i32) -> Self {
        Self {
            race,
            hp: (hp, hp as u32),
            speed: 4,
            damage: 1,
            range: 3,
        }
    }

    pub fn harm(&mut self, x: u32) {
        self.hp.0 -= x as i32;
    }

    pub fn race(&self) -> Race {
        self.race
    }
    pub fn hp(&self) -> i32 {
        self.hp.0
    }
    pub fn damage(&self) -> u32 {
        self.damage
    }
    pub fn speed(&self) -> u32 {
        self.speed
    }
    pub fn range(&self) -> u32 {
        self.range
    }
}
