use ::core::convert::Infallible;
use ::core::fmt::{Debug, Display, Formatter};
use std::error::Error;
use std::ffi::c_uint;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MemoryType(c_uint);

impl MemoryType {
  pub fn new(n: c_uint) -> Self {
    Self(n)
  }

  pub fn into_inner(self) -> c_uint {
    self.0
  }
}

impl From<c_uint> for MemoryType {
  fn from(n: c_uint) -> Self {
    Self(n)
  }
}

impl From<MemoryType> for c_uint {
  fn from(memory_type: MemoryType) -> Self {
    memory_type.into_inner()
  }
}

trait TypeId: Sized {
  fn into_discriminant(self) -> u8;
  fn from_discriminant(id: u8) -> Option<Self>;
}

impl TypeId for () {
  fn into_discriminant(self) -> u8 {
    0
  }

  fn from_discriminant(_id: u8) -> Option<Self> {
    None
  }
}

/// Enum for the `RETRO_MEMORY_*` constants in `libretro.h`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum StandardMemoryType {
  #[default]
  SaveRam = 0,
  RTC = 1,
  SystemRam = 2,
  VideoRam = 3,
}

impl Display for StandardMemoryType {
  fn fmt(&self, f: &mut Formatter<'_>) -> ::core::fmt::Result {
    Display::fmt(&(*self as u8), f)
  }
}

impl From<StandardMemoryType> for MemoryType {
  /// Converts the standard memory types back into their constants, and
  /// left-shifts subsystem memory types to the upper 8 bits as recommended
  /// by the libretro API to avoid conflicts with future memory types.
  fn from(mem_type: StandardMemoryType) -> Self {
    MemoryType::new(mem_type as c_uint)
  }
}

impl TryFrom<MemoryType> for StandardMemoryType {
  type Error = TryFromRetroMemoryTypeError;

  fn try_from(mem_type: MemoryType) -> Result<Self, Self::Error> {
    match mem_type.into_inner() {
      0 => Ok(Self::SaveRam),
      1 => Ok(Self::RTC),
      2 => Ok(Self::SystemRam),
      3 => Ok(Self::VideoRam),
      _ => Err(TryFromRetroMemoryTypeError(())),
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// Based on std::num::TryFromIntError.
// The crate-private field prevents use of the constructor outside the crate.
pub struct TryFromRetroMemoryTypeError(pub(crate) ());

impl Display for TryFromRetroMemoryTypeError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "attempted to convert an unknown memory type")
  }
}

impl Error for TryFromRetroMemoryTypeError {}

impl From<Infallible> for TryFromRetroMemoryTypeError {
  fn from(x: Infallible) -> Self {
    match x {}
  }
}
