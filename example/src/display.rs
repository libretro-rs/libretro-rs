pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;
pub const AREA: usize = WIDTH * HEIGHT;

const WIDTH_MASK: usize = WIDTH - 1;
const HEIGHT_MASK: usize = HEIGHT - 1;

#[derive(Clone, Copy)]
pub enum Pixel {
  Off,
  On,
}

impl Pixel {
  pub fn invert(self) -> Pixel {
    match self {
      Pixel::Off => Pixel::On,
      Pixel::On => Pixel::Off,
    }
  }
}

pub struct Display {
  buffer: [[Pixel; WIDTH]; HEIGHT],
}

impl Display {
  pub fn new() -> Display {
    Display {
      buffer: [[Pixel::Off; WIDTH]; HEIGHT],
    }
  }

  pub fn pixel(&self, x: usize, y: usize) -> Pixel {
    self.buffer[y & HEIGHT_MASK][x & WIDTH_MASK]
  }

  pub fn set_pixel(&mut self, x: usize, y: usize, pixel: Pixel) {
    self.buffer[y & HEIGHT_MASK][x & WIDTH_MASK] = pixel
  }

  /// Handler for the `cls` instruction.
  pub fn cls(&mut self) {
    for y in 0..HEIGHT {
      for x in 0..WIDTH {
        self.set_pixel(x, y, Pixel::Off);
      }
    }
  }

  /// Handler for the `drw` instruction.
  pub fn drw(&mut self, x: usize, y: usize, sprite_data: &[(usize, usize)]) -> bool {
    let mut collision = false;

    for (row, tile) in sprite_data {
      for col in 0..7 {
        let pixel = (tile >> (7 - col)) & 1;
        if pixel == 1 {
          let previous = self.pixel(col + x, row + y);
          if let Pixel::On = previous {
            collision = true;
          }

          self.set_pixel(col + x, row + y, previous.invert())
        }
      }
    }

    collision
  }
}
