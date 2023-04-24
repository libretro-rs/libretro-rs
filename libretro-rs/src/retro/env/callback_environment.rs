use super::*;
use crate::retro::*;
use ::core::ffi::*;

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct EnvironmentPtr(non_null_retro_environment_t);

impl EnvironmentPtr {
  pub fn new(ptr: non_null_retro_environment_t) -> Self {
    Self(ptr)
  }

  unsafe fn with_ref(&self, cmd: c_uint, data: &impl CommandData) -> env::Result<()> {
    if self.0(cmd, data as *const _ as *mut c_void) {
      Ok(())
    } else {
      Err(CommandError::new())
    }
  }

  unsafe fn with_mut(&self, cmd: c_uint, data: &mut impl CommandData) -> env::Result<()> {
    if self.0(cmd, data as *mut _ as *mut c_void) {
      Ok(())
    } else {
      Err(CommandError::new())
    }
  }
}

impl EnvironmentCallback for EnvironmentPtr {
  unsafe fn get(&self, cmd: c_uint, data: &mut impl CommandData) -> env::Result<()> {
    self.with_mut(cmd, data)
  }

  unsafe fn set(&mut self, cmd: c_uint, data: &impl CommandData) -> env::Result<()> {
    self.with_ref(cmd, data)
  }

  unsafe fn cmd(&mut self, cmd: c_uint, data: &mut impl CommandData) -> env::Result<()> {
    self.with_mut(cmd, data)
  }
}
