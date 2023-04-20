use crate::prelude::*;
use crate::retro::environment::Result;
use core::borrow::{Borrow, BorrowMut};
use core::ffi::c_void;

pub type EnvironmentCallback = unsafe extern "C" fn(cmd: u32, data: *mut c_void) -> bool;

impl Environment for EnvironmentCallback {
  unsafe fn parameterized_get_raw<C, D, R>(&self, cmd: C, data: D) -> Result<R>
  where
    C: Into<u32>,
    D: Into<R>,
    R: CommandData,
  {
    let mut data = data.into();
    callback_mut(self, cmd, &mut data).map(|_| data)
  }

  unsafe fn set_raw<C, D, R>(&mut self, cmd: C, data: &D) -> Result<()>
  where
    C: Into<u32>,
    D: Borrow<R>,
    R: CommandData,
  {
    callback_ref(self, cmd, data.borrow())
  }

  unsafe fn parameterized_cmd_raw<C, D, R>(&mut self, cmd: C, data: &mut D) -> Result<()>
  where
    C: Into<u32>,
    D: BorrowMut<R>,
    R: CommandData,
  {
    callback_mut(self, cmd, data.borrow_mut())
  }
}

unsafe fn callback_ref<C, D>(cb: &EnvironmentCallback, cmd: C, data: &D) -> Result<()>
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

unsafe fn callback_mut<C, D>(cb: &EnvironmentCallback, cmd: C, data: &mut D) -> Result<()>
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
