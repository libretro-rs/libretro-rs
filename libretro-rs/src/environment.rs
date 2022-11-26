use crate::*;
use core::ffi::*;
use core::mem::MaybeUninit;
use libretro_rs_sys::*;

impl RetroEnvironment for EnvironmentCallback {
  unsafe fn get_raw<T>(&self, key: u32) -> Option<T> where T: Copy {
    let mut value = MaybeUninit::uninit();
    if self(key, value.as_mut_ptr() as *mut _ as *mut c_void) {
      unsafe { Some(value.assume_init()) }
    } else {
      None
    }
  }

  unsafe fn set_raw<T>(&mut self, key: u32, val: T) -> bool where T: Copy {
    self(key, &val as *const _ as *mut c_void)
  }

  unsafe fn cmd_raw(&mut self, key: u32) -> bool {
    self(key, core::ptr::null_mut())
  }

  unsafe fn mut_struct_raw<T>(&mut self, key: u32, val: &mut T) -> bool {
    self(key, val as *mut _ as *mut c_void)
  }

  unsafe fn set_struct_raw<T>(&mut self, key: u32, val: &T) -> bool {
    self(key, val as *const _ as *mut c_void)
  }
}

/// Exposes the [retro_environment_t] callback in an idiomatic fashion. Each of the `RETRO_ENVIRONMENT_*` keys will
/// eventually have a corresponding method here.
///
/// Until that is accomplished, the keys are available in [libretro_rs::sys] and can be used manually with the various `*_raw` methods.
pub trait RetroEnvironment: Sized {
  /// Boolean value indicating whether or not frontend supports frame duping.
  fn get_can_dupe(&self) -> bool {
    unsafe { self.get_bool(RETRO_ENVIRONMENT_GET_CAN_DUPE) }
  }

  /// Queries the path where the current libretro core resides.
  fn get_libretro_path(&self) -> Option<&str> {
    unsafe { self.get_str(RETRO_ENVIRONMENT_GET_LIBRETRO_PATH) }
  }

  /// Queries the path of the "core assets" directory.
  fn get_core_assets_directory(&self) -> Option<&str> {
    unsafe { self.get_str(RETRO_ENVIRONMENT_GET_CORE_ASSETS_DIRECTORY) }
  }

  /// Queries the path of the save directory.
  fn get_save_directory(&self) -> Option<&str> {
    unsafe { self.get_str(RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY) }
  }

  /// Queries the path of the system directory.
  fn get_system_directory(&self) -> Option<&str> {
    unsafe { self.get_str(RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY) }
  }

  /// Queries the username associated with the frontend.
  fn get_username(&self) -> Option<&str> {
    unsafe { self.get_str(RETRO_ENVIRONMENT_GET_USERNAME) }
  }

  /// Convenience method for querying a [&str] value with [get_copyable_raw].
  unsafe fn get_str(&self, key: u32) -> Option<&str> {
    self.get_raw::<*const c_char>(key)
      .map(|ptr| CStr::from_ptr(ptr).to_str().unwrap())
  }

  /// Gets a [bool] value with [get_copyable_raw], translating [None] to [false].
  unsafe fn get_bool(&self, key: u32) -> bool {
    self.get_raw(key).unwrap_or(false)
  }

  /// Directly invokes the underlying [retro_environment_t] in a "get" fashion.
  ///
  /// # Safety
  /// Using the environment callback in a way that violates the libretro specification is unsafe.
  unsafe fn get_raw<T>(&self, key: u32) -> Option<T> where T: Copy;

  /// Directly invokes the underlying [retro_environment_t] in a "set" fashion.
  ///
  /// # Safety
  /// Using the environment callback in a way that violates the libretro specification is unsafe.
  unsafe fn set_raw<T>(&mut self, key: u32, val: T) -> bool where T: Copy;

  /// Directly invokes the underlying [retro_environment_t] in a "command" fashion.
  ///
  /// # Safety
  /// Using the environment callback in a way that violates the libretro specification is unsafe.
  unsafe fn cmd_raw(&mut self, key: u32) -> bool;

  /// Directly invokes the underlying [retro_environment_t] in a read-write fashion.
  ///
  /// # Safety
  /// Using the environment callback in a way that violates the libretro specification is unsafe.
  unsafe fn mut_struct_raw<T>(&mut self, key: u32, val: &mut T) -> bool;

  /// Directly invokes the underlying [retro_environment_t] in a "set" fashion.
  ///
  /// # Safety
  /// Using the environment callback in a way that violates the libretro specification is unsafe.
  unsafe fn set_struct_raw<T>(&mut self, key: u32, val: &T) -> bool;
}

pub trait SetEnvironmentEnvironment: RetroEnvironment {
  fn set_support_no_game(&mut self, val: bool) -> bool {
    unsafe { self.set_raw(RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME, &val) }
  }
}
impl <T> SetEnvironmentEnvironment for T where T: RetroEnvironment {}

pub trait InitEnvironment: RetroEnvironment {}
impl <T> InitEnvironment for T where T: RetroEnvironment {}

pub trait SetPortDeviceEnvironment: RetroEnvironment {}
impl <T> SetPortDeviceEnvironment for T where T: RetroEnvironment {}

pub trait ResetEnvironment: RetroEnvironment {}
impl <T> ResetEnvironment for T where T: RetroEnvironment {}

pub trait RunEnvironment: RetroEnvironment {
  /// Requests that the frontend shut down. The frontend can refuse to do this, and return false.
  fn shutdown(&mut self) -> bool {
    unsafe { self.cmd_raw(RETRO_ENVIRONMENT_SHUTDOWN) }
  }

  fn set_geometry(&mut self, val: RetroGameGeometry) -> bool {
    let val: retro_game_geometry = val.into();
    unsafe { self.set_raw(RETRO_ENVIRONMENT_SET_GEOMETRY, &val) }
  }
}
impl <T> RunEnvironment for T where T: RetroEnvironment {}

pub trait SerializeSizeEnvironment: RetroEnvironment {}
impl <T> SerializeSizeEnvironment for T where T: RetroEnvironment {}

pub trait SerializeEnvironment: RetroEnvironment {}
impl <T> SerializeEnvironment for T where T: RetroEnvironment {}

pub trait UnserializeEnvironment: RetroEnvironment {}
impl <T> UnserializeEnvironment for T where T: RetroEnvironment {}

pub trait CheatResetEnvironment: RetroEnvironment {}
impl <T> CheatResetEnvironment for T where T: RetroEnvironment {}

pub trait CheatSetEnvironment: RetroEnvironment {}
impl <T> CheatSetEnvironment for T where T: RetroEnvironment {}

pub trait LoadGameEnvironment: RetroEnvironment {
  fn set_pixel_format(&mut self, format: RetroPixelFormat) -> bool {
    GetSystemAvInfoEnvironment::set_pixel_format(self, format)
  }
}
impl <T> LoadGameEnvironment for T where T: RetroEnvironment {}

pub trait GetSystemAvInfoEnvironment: RetroEnvironment {
  fn set_pixel_format(&mut self, format: RetroPixelFormat) -> bool {
    unsafe { self.set_raw(RETRO_ENVIRONMENT_SET_PIXEL_FORMAT, &format) }
  }
}
impl <T> GetSystemAvInfoEnvironment for T where T: RetroEnvironment {}

pub trait GetRegionEnvironment: RetroEnvironment {}
impl <T> GetRegionEnvironment for T where T: RetroEnvironment {}

pub trait LoadGameSpecialEnvironment: RetroEnvironment {}
impl <T> LoadGameSpecialEnvironment for T where T: RetroEnvironment {}

pub trait UnloadGameEnvironment: RetroEnvironment {}
impl <T> UnloadGameEnvironment for T where T: RetroEnvironment {}

pub trait GetMemoryDataEnvironment: RetroEnvironment {}
impl <T> GetMemoryDataEnvironment for T where T: RetroEnvironment {}

pub trait GetMemorySizeEnvironment: RetroEnvironment {}
impl <T> GetMemorySizeEnvironment for T where T: RetroEnvironment {}
