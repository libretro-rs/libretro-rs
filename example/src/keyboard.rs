const MAX_KEYS: usize = 16;

#[derive(Clone, Copy, Default)]
pub struct Key(usize);

impl Key {
  pub fn new(val: usize) -> Key {
    Key(val)
  }

  pub fn ordinal(&self) -> u8 {
    self.0 as u8
  }
}

#[derive(Clone, Copy)]
pub enum KeyState {
  Released,
  Pressed,
}

impl KeyState {
  pub fn is_pressed(self) -> bool {
    match self {
      KeyState::Released => false,
      KeyState::Pressed => true,
    }
  }
}

pub struct Keyboard {
  key_states: [KeyState; MAX_KEYS],
}

impl Keyboard {
  pub fn new() -> Keyboard {
    Keyboard {
      key_states: [KeyState::Released; MAX_KEYS],
    }
  }

  pub fn key_state(&self, key: Key) -> KeyState {
    self.key_states[key.0]
  }

  pub fn set_key_state(&mut self, key: Key, state: KeyState) {
    self.key_states[key.0] = state
  }

  pub fn first_pressed_key(&self) -> Option<Key> {
    Self::keys().find(|&key| self.key_state(key).is_pressed())
  }

  pub fn keys() -> impl Iterator<Item = Key> {
    (0..MAX_KEYS).map(Key)
  }
}
