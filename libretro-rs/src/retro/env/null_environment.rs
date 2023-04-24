use super::*;
use crate::retro::*;
use ::core::ffi::*;

/// A [Environment] that doesn't implement any commands. Useful for testing.
#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct NullEnvironment(pub(crate) ());

impl NullEnvironment {
  pub fn new() -> Self {
    Self(())
  }
}

impl EnvironmentCallback for NullEnvironment {
  unsafe fn get(&self, _cmd: c_uint, _data: &mut impl CommandData) -> env::Result<()> {
    Err(CommandError::new())
  }

  unsafe fn set(&mut self, _cmd: c_uint, _data: &impl CommandData) -> env::Result<()> {
    Err(CommandError::new())
  }

  unsafe fn cmd(&mut self, _cmd: c_uint, _data: &mut impl CommandData) -> env::Result<()> {
    Err(CommandError::new())
  }
}
