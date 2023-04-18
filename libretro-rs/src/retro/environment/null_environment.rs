use crate::retro::environment::convert::*;
use crate::retro::*;

/// A [Environment] that doesn't implement any commands. Useful for testing.
pub struct NullEnvironment;

impl Environment for NullEnvironment {
  unsafe fn parameterized_get_raw<T>(&self, _cmd: impl Into<u32>, _data: impl Into<T::Source>) -> T
  where
    T: EnvironmentResult,
  {
    T::unsafe_from(None)
  }

  unsafe fn set_raw(&mut self, _cmd: impl Into<u32>, _data: &impl EnvironmentData) -> bool {
    false
  }

  unsafe fn parameterized_cmd_raw<T>(&mut self, _cmd: impl Into<u32>, _data: impl Into<T::Source>) -> T
  where
    T: EnvironmentResult,
  {
    T::unsafe_from(None)
  }
}
