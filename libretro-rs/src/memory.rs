/// Enum for the `RETRO_MEMORY_*` constants in `libretro.h`, as well as
/// user-defined subsystem memory types.
pub enum RetroMemoryType<T> {
  SaveRam,
  RTC,
  SystemRam,
  VideoRam,
  Subsystem(T),
}

impl<T> From<RetroMemoryType<T>> for u16
where
  T: Into<u8>,
{
  /// Converts the standard memory types back into their constants, and
  /// left-shifts subsystem memory types to the upper 8 bits as recommended
  /// by the libretro API to avoid conflicts with future memory types.
  fn from(mem_type: RetroMemoryType<T>) -> Self {
    use RetroMemoryType::*;
    match mem_type {
      SaveRam => 0,
      RTC => 1,
      SystemRam => 2,
      VideoRam => 3,
      Subsystem(subsystem_type) => (subsystem_type.into() as u16) << 8,
    }
  }
}

impl<T> TryFrom<u16> for RetroMemoryType<T>
where
  T: TryFrom<u8>,
{
  type Error = &'static str;

  /// Attempts to convert a [u16] into a known [RetroMemoryType].
  fn try_from(mem_type: u16) -> Result<Self, Self::Error> {
    use RetroMemoryType::*;
    match mem_type {
      0 => Ok(SaveRam),
      1 => Ok(RTC),
      2 => Ok(SystemRam),
      3 => Ok(VideoRam),
      _ => {
        if mem_type < 256 {
          Err("Unknown standard memory type")
        } else {
          T::try_from((mem_type >> 8) as u8)
            .map(|mem_type| Subsystem(mem_type))
            .map_err(|_| "Unknown subsystem memory type")
        }
      }
    }
  }
}
