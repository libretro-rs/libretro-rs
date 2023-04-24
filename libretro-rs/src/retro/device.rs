use ::core::ffi::*;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// A numeric ID for a libretro device type provided by the frontend.
/// An enum is also provided for the standard types listed in the libretro API:
/// see [`DeviceType`].
pub struct DeviceTypeId(c_uint);

impl DeviceTypeId {
  pub fn new(id: c_uint) -> Self {
    Self(id)
  }

  pub fn into_inner(self) -> c_uint {
    self.0
  }
}

impl From<c_uint> for DeviceTypeId {
  fn from(port_number: c_uint) -> Self {
    Self::new(port_number)
  }
}

impl From<DeviceTypeId> for c_uint {
  fn from(id: DeviceTypeId) -> Self {
    id.into_inner()
  }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DeviceType {
  #[default]
  None = 0,
  Joypad = 1,
  Mouse = 2,
  Keyboard = 3,
  LightGun = 4,
  Analog = 5,
  Pointer = 6,
}

impl TryFrom<DeviceTypeId> for DeviceType {
  type Error = ();

  fn try_from(val: DeviceTypeId) -> Result<Self, Self::Error> {
    match val.into_inner() {
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DevicePort(c_uint);

impl DevicePort {
  /// Creates a [`DevicePort`].
  pub fn new(port_number: c_uint) -> Self {
    DevicePort(port_number)
  }

  // Converts this port back into a u8.
  pub fn into_inner(self) -> c_uint {
    self.0
  }
}

impl From<c_uint> for DevicePort {
  fn from(port_number: c_uint) -> Self {
    Self::new(port_number)
  }
}

impl From<DevicePort> for c_uint {
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

impl From<JoypadButton> for c_uint {
  fn from(button: JoypadButton) -> c_uint {
    button as c_uint
  }
}
