use crate::retro::*;
use env::{CommandData, EnvironmentCallback, Result};
use std::ffi::c_uint;

/// A [Environment] that doesn't implement any commands. Useful for testing.
#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct NullEnvironment(pub(crate) ());

impl NullEnvironment {
  pub fn new() -> Self {
    Self(())
  }
}

impl EnvironmentCallback for NullEnvironment {
  unsafe fn get(&self, _cmd: c_uint, _data: &mut impl CommandData) -> Result<()> {
    Err(CommandError::new())
  }

  unsafe fn set(&mut self, _cmd: c_uint, _data: &impl CommandData) -> Result<()> {
    Err(CommandError::new())
  }

  unsafe fn cmd(&mut self, _cmd: c_uint, _data: &mut impl CommandData) -> Result<()> {
    Err(CommandError::new())
  }
}
