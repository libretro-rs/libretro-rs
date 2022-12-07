use crate::environment::convert::*;
use crate::*;

/// A [RetroEnvironment] that doesn't implement any commands. Useful for testing.
pub struct NullEnvironment;

impl RetroEnvironment for NullEnvironment {
  unsafe fn parameterized_get_raw<T>(&self, _cmd: impl Into<u32>, _data: impl Into<T::Source>) -> T
  where
    T: RetroEnvironmentResult,
  {
    T::unsafe_from(None)
  }

  unsafe fn set_raw(&mut self, _cmd: impl Into<u32>, _data: &impl RetroEnvironmentData) -> bool {
    false
  }

  unsafe fn parameterized_cmd_raw<T>(&mut self, _cmd: impl Into<u32>, _data: impl Into<T::Source>) -> T
  where
    T: RetroEnvironmentResult,
  {
    T::unsafe_from(None)
  }
}
