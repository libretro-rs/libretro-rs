use crate::retro::*;

pub type EnvironmentCallback = unsafe extern "C" fn(cmd: u32, data: *mut c_void) -> bool;

impl Environment for EnvironmentCallback {
  unsafe fn parameterized_get_raw<T>(&self, cmd: impl Into<u32>, data: impl Into<T::Source>) -> T
  where
    T: EnvironmentResult,
  {
    get_option_from_callback(self, cmd, data.into())
  }

  unsafe fn set_raw(&mut self, cmd: impl Into<u32>, data: &impl EnvironmentData) -> bool {
    self(cmd.into(), data as *const _ as *mut c_void)
  }

  unsafe fn parameterized_cmd_raw<T>(&mut self, cmd: impl Into<u32>, data: impl Into<T::Source>) -> T
  where
    T: EnvironmentResult,
  {
    get_option_from_callback(self, cmd, data.into())
  }
}

unsafe fn get_option_from_callback<T, U>(cb: &EnvironmentCallback, cmd: impl Into<u32>, mut data: U) -> T
where
  T: EnvironmentResult<Source = U>,
  U: EnvironmentData,
{
  T::unsafe_from(if cb(cmd.into(), &mut data as *mut _ as *mut c_void) {
    Some(data)
  } else {
    None
  })
}