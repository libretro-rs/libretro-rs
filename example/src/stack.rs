const MAX_CAPACITY: usize = 16;

pub struct Stack {
  index: usize,
  array: [u16; MAX_CAPACITY],
}

impl Stack {
  pub fn new() -> Stack {
    Stack {
      index: 0,
      array: [0; MAX_CAPACITY],
    }
  }

  pub fn pull(&mut self) -> u16 {
    assert!(self.index > 0, "tried to pull from an empty stack");
    self.index -= 1;
    self.array[self.index]
  }

  pub fn push(&mut self, value: u16) {
    assert!(self.index < MAX_CAPACITY, "tried to push onto a full stack");
    self.array[self.index] = value;
    self.index += 1;
  }
}
