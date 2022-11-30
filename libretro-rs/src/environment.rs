use crate::*;
use core::ffi::*;
use core::mem::*;
use libretro_rs_sys::*;

impl RetroEnvironment for EnvironmentCallback {
  unsafe fn get_raw<T>(&self, key: u32) -> Option<T> where T: Copy {
    let mut data = MaybeUninit::uninit();
    if self(key, data.as_mut_ptr() as *mut _ as *mut c_void) {
      unsafe { Some(data.assume_init()) }
    } else {
      None
    }
  }

  unsafe fn set_raw<T>(&mut self, key: u32, data: &T) -> bool {
    self(key, &data as *const _ as *mut c_void)
  }

  unsafe fn cmd_raw(&mut self, key: u32) -> bool {
    self(key, core::ptr::null_mut())
  }

  unsafe fn mut_ref_raw<T>(&mut self, key: u32, data: &mut T) -> bool {
    self(key, data as *mut _ as *mut c_void)
  }
}

/// Exposes the [retro_environment_t] callback in an idiomatic fashion. Each of the `RETRO_ENVIRONMENT_*` keys will
/// eventually have a corresponding method here.
///
/// Until that is accomplished, the keys are available in [libretro_rs::sys] and can be used manually with the various `*_raw` methods.
pub trait RetroEnvironment: Sized {
  /// Boolean value indicating whether or not frontend supports frame duping.
  fn get_can_dupe(&self) -> bool {
    unsafe { self.get_raw(RETRO_ENVIRONMENT_GET_CAN_DUPE).unwrap_or(false) }
  }

  /// Queries the path where the current libretro core resides.
  fn get_libretro_path(&self) -> OptionCStr {
    unsafe { self.get_c_str_raw(RETRO_ENVIRONMENT_GET_LIBRETRO_PATH) }
  }

  /// Queries the path of the "core assets" directory.
  fn get_core_assets_directory(&self) -> OptionCStr {
    unsafe { self.get_c_str_raw(RETRO_ENVIRONMENT_GET_CORE_ASSETS_DIRECTORY) }
  }

  /// Queries the path of the save directory.
  fn get_save_directory(&self) -> OptionCStr {
    unsafe { self.get_c_str_raw(RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY) }
  }

  /// Queries the path of the system directory.
  fn get_system_directory(&self) -> OptionCStr {
    unsafe { self.get_c_str_raw(RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY) }
  }

  /// Queries the username associated with the frontend.
  fn get_username(&self) -> OptionCStr {
    unsafe { self.get_c_str_raw(RETRO_ENVIRONMENT_GET_USERNAME) }
  }

  /// Convenience method for querying a [CStr] value with [Self::get_raw].
  unsafe fn get_c_str_raw(&self, key: u32) -> OptionCStr {
    OptionCStr(self.get_raw(key).map(|ptr| CStr::from_ptr(ptr)))
  }

  /// Directly invokes the underlying [retro_environment_t] in a "get" fashion.
  /// To get a struct, see [Self::mut_struct_raw].
  ///
  /// # Safety
  /// The environment command **must** return a value when successful.
  /// Using the environment callback in a way that violates the libretro specification is unsafe.
  unsafe fn get_raw<T>(&self, key: u32) -> Option<T> where T: Copy;

  /// Directly invokes the underlying [retro_environment_t] in a "set" fashion.
  /// To set a struct, see [Self::set_struct_raw].
  ///
  /// # Safety
  /// The environment command must **not** mutate `data`.
  /// Using the environment callback in a way that violates the libretro specification is unsafe.
  unsafe fn set_raw<T>(&mut self, key: u32, data: &T) -> bool;

  /// Directly invokes the underlying [retro_environment_t] in a "command" fashion.
  ///
  /// # Safety
  /// Using the environment callback in a way that violates the libretro specification is unsafe.
  unsafe fn cmd_raw(&mut self, key: u32) -> bool;

  /// Directly invokes the underlying [retro_environment_t] in a read-write fashion.
  ///
  /// # Safety
  /// Using the environment callback in a way that violates the libretro specification is unsafe.
  unsafe fn mut_ref_raw<T>(&mut self, key: u32, data: &mut T) -> bool;
}

pub trait SetEnvironmentEnvironment: RetroEnvironment {
  fn set_support_no_game(&mut self, data: bool) -> bool {
    unsafe { self.set_raw(RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME, &data) }
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

  fn set_geometry(&mut self, geometry: RetroGameGeometry) -> bool {
    let data: retro_game_geometry = geometry.into();
    unsafe { self.set_raw(RETRO_ENVIRONMENT_SET_GEOMETRY, &data) }
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
    unsafe { self.set_raw(RETRO_ENVIRONMENT_SET_PIXEL_FORMAT, &u32::from(format)) }
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
