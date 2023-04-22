use ::core::ffi::*;

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Device {
  #[default]
  None = 0,
  Joypad = 1,
  Mouse = 2,
  Keyboard = 3,
  LightGun = 4,
  Analog = 5,
  Pointer = 6,
}

impl TryFrom<c_uint> for Device {
  type Error = ();

  fn try_from(val: c_uint) -> Result<Self, Self::Error> {
    match val {
      0 => Ok(Self::None),
      1 => Ok(Self::Joypad),
      2 => Ok(Self::Mouse),
      3 => Ok(Self::Keyboard),
      4 => Ok(Self::LightGun),
      5 => Ok(Self::Analog),
      6 => Ok(Self::Pointer),
      _ => Err(()),
    }
  }
}

/// A libretro device port.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DevicePort(u8);

impl DevicePort {
  /// Creates a [`DevicePort`].
  pub fn new(port_number: u8) -> Self {
    DevicePort(port_number)
  }

  // Converts this port back into a u8.
  pub fn into_inner(self) -> u8 {
    self.0
  }
}

impl From<u8> for DevicePort {
  fn from(port_number: u8) -> Self {
    Self::new(port_number)
  }
}

impl From<DevicePort> for u8 {
  fn from(port: DevicePort) -> Self {
    port.into_inner()
  }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum JoypadButton {
  #[default]
  B = 0,
  Y = 1,
  Select = 2,
  Start = 3,
  Up = 4,
  Down = 5,
  Left = 6,
  Right = 7,
  A = 8,
  X = 9,
  L1 = 10,
  R1 = 11,
  L2 = 12,
  R2 = 13,
  L3 = 14,
  R3 = 15,
  #[cfg(experimental)]
  Mask = 256,
}

impl From<JoypadButton> for u32 {
  fn from(button: JoypadButton) -> u32 {
    match button {
      JoypadButton::B => 0,
      JoypadButton::Y => 1,
      JoypadButton::Select => 2,
      JoypadButton::Start => 3,
      JoypadButton::Up => 4,
      JoypadButton::Down => 5,
      JoypadButton::Left => 6,
      JoypadButton::Right => 7,
      JoypadButton::A => 8,
      JoypadButton::X => 9,
      JoypadButton::L1 => 10,
      JoypadButton::R1 => 11,
      JoypadButton::L2 => 12,
      JoypadButton::R2 => 13,
      JoypadButton::L3 => 14,
      JoypadButton::R3 => 15,
      #[cfg(experimental)]
      JoypadButton::Mask => 256,
    }
  }
}
