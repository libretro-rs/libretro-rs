pub const SAMPLE_FREQUENCY: usize = 44_100;
pub const AUDIO_BUFFER_SIZE: usize = SAMPLE_FREQUENCY / 60;
pub const AUDIO_FREQUENCY: f64 = 440.0;
pub const RATE: f64 = (std::f64::consts::TAU * AUDIO_FREQUENCY) / (SAMPLE_FREQUENCY as f64);

pub struct Timer {
  delay: i32,
  sound: i32,
  level: f64,
}

impl Timer {
  pub fn new() -> Timer {
    Timer {
      delay: 0,
      sound: 0,
      level: 0.0,
    }
  }

  pub fn delay(&self) -> i32 {
    self.delay
  }

  pub fn set_delay(&mut self, value: i32) {
    self.delay = value;
  }

  pub fn set_sound(&mut self, value: i32) {
    self.sound = value;
  }

  pub fn update(&mut self) {
    let delay = (self.delay - 1).max(0);
    self.delay = delay;
    self.sound = delay;
  }

  pub fn wave(&mut self, mut cb: impl FnMut(usize, f64)) {
    for n in 0..AUDIO_BUFFER_SIZE {
      cb(n, self.level.sin() / 2.0);

      if self.sound > 0 {
        self.level = (self.level + RATE) % std::f64::consts::TAU;
      }
    }
  }
}
