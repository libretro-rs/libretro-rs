use rand::prelude::*;

pub struct Random(ThreadRng);

impl Random {
  pub fn new() -> Random {
    Random(rand::thread_rng())
  }

  pub fn next(&mut self, mask: u8) -> u8 {
    let number: u8 = self.0.gen();
    number & mask
  }
}
