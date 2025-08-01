mod player;

pub use player::Player;

#[derive(Default)]
pub struct Cooldown {
    pub value: bool,
    delay: f32,
}

impl Cooldown {
    pub fn new() -> Self {
        Self {
            value: false,
            delay: 0.0,
        }
    }

    pub fn set(&mut self, delay: f32) {
        self.delay = delay;
        self.value = true;
    }

    pub fn update(&mut self, delta: f32) {
        if !self.value {
            return;
        }
        if delta > self.delay {
            self.delay = 0.0;
            self.value = false;
        } else {
            self.delay -= delta;
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub enum State {
    WALK,
    STAND,
    #[default]
    FALL,
    ALERT,
    PRONE,
    SWIM,
    CLIMB,
    DIED,
    SIT,
}