use crate::prelude::*;
use crate::retro::environment::convert::*;
use crate::retro::environment::Result;

/// A [Environment] that doesn't implement any commands. Useful for testing.
pub struct NullEnvironment;

impl Environment for NullEnvironment {
  unsafe fn parameterized_get_raw<C, D, R>(&self, _cmd: C, _data: D) -> Result<R>
  where
    C: Into<u32>,
    D: Into<R>,
    R: CommandData,
  {
    Err(CommandError::new())
  }

  unsafe fn set_raw<C, D, R>(&mut self, _cmd: C, _data: &D) -> Result<()>
  where
    C: Into<u32>,
    D: core::borrow::Borrow<R>,
    R: CommandData,
  {
    Err(CommandError::new())
  }

  unsafe fn parameterized_cmd_raw<C, D, R>(&mut self, _cmd: C, _data: &mut D) -> Result<()>
  where
    C: Into<u32>,
    D: core::borrow::BorrowMut<R>,
    R: CommandData,
  {
    Err(CommandError::new())
  }
}
