use crate::retro::*;
use env::{CommandData, EnvironmentCallback, Result};

/// A [Environment] that doesn't implement any commands. Useful for testing.
pub struct NullEnvironment;

impl EnvironmentCallback for NullEnvironment {
  unsafe fn get(&self, _cmd: u32, _data: &mut impl CommandData) -> Result<()> {
    Err(CommandError::new())
  }

  unsafe fn set(&mut self, _cmd: u32, _data: &impl CommandData) -> Result<()> {
    Err(CommandError::new())
  }

  unsafe fn cmd(&mut self, _cmd: u32, _data: &mut impl CommandData) -> Result<()> {
    Err(CommandError::new())
  }
}
