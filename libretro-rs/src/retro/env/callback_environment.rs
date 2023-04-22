use super::*;
use crate::retro::*;
use ::core::ffi::*;

pub type EnvironmentPtr = unsafe extern "C" fn(cmd: u32, data: *mut c_void) -> bool;

impl EnvironmentCallback for EnvironmentPtr {
  unsafe fn get(&self, cmd: u32, data: &mut impl CommandData) -> Result<()> {
    callback_mut(*self, cmd, data)
  }

  unsafe fn set(&mut self, cmd: u32, data: &impl CommandData) -> Result<()> {
    callback_ref(self, cmd, data)
  }

  unsafe fn cmd(&mut self, cmd: u32, data: &mut impl CommandData) -> Result<()> {
    callback_mut(*self, cmd, data)
  }
}

unsafe fn callback_ref<C, D>(cb: &EnvironmentPtr, cmd: C, data: &D) -> Result<()>
where
  C: Into<u32>,
  D: CommandData,
{
  if cb(cmd.into(), data as *const _ as *mut c_void) {
    Ok(())
  } else {
    Err(CommandError::new())
  }
}

unsafe fn callback_mut<C, D>(cb: EnvironmentPtr, cmd: C, data: &mut D) -> Result<()>
where
  C: Into<u32>,
  D: CommandData,
{
  if cb(cmd.into(), data as *mut _ as *mut c_void) {
    Ok(())
  } else {
    Err(CommandError::new())
  }
}
