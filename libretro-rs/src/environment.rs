use crate::*;
use core::ffi::*;
use libretro_rs_sys::*;

/// Marker trait for types that are valid arguments to the environment callback.
///
/// Any type implementing this trait must be FFI-safe. Structs should be `#[repr(C)]` or a
/// `#[repr(transparent)]` newtype. Numeric enums should have the appropriate primitive
/// representation, which is typically either `#[repr(u32)]` for `const unsigned` arguments or
/// `#[repr(i32)]` for `const enum` arguments.
///
/// Care must still be taken when calling any of the generic unsafe `[RetroEnvironment]` methods to
/// ensure the type used is appropriate for the environment command, as specified in `libretro.h`.
pub trait RetroEnvironmentData {}
impl RetroEnvironmentData for bool {}
impl RetroEnvironmentData for Option<&c_char> {}
impl RetroEnvironmentData for retro_log_callback {}
impl RetroEnvironmentData for RetroMessage {}
impl RetroEnvironmentData for RetroPixelFormat {}
impl RetroEnvironmentData for RetroGameGeometry {}
impl RetroEnvironmentData for ScreenRotation {}
impl RetroEnvironmentData for u32 {}
impl<T> RetroEnvironmentData for Option<T> where T: RetroEnvironmentData {}

pub type EnvironmentCallback = unsafe extern "C" fn(cmd: u32, data: *mut c_void) -> bool;

/// A [RetroEnvironment] that doesn't implement any commands. Useful for testing.
pub struct NullEnvironment;

/// Exposes the [retro_environment_t] callback in an idiomatic fashion.
/// Each of the `RETRO_ENVIRONMENT_*` keys will eventually have a corresponding method here.
///
/// Until that is accomplished, the keys are available in [libretro_rs::sys] and can be used
/// manually with the various `*_raw` methods.
pub trait RetroEnvironment: Sized {
  /// Directly invokes the underlying [retro_environment_t] in a "get" fashion.
  /// Returns `Some(T)` iff the command succeeds.
  ///
  /// # Safety
  /// See `libretro.h` for the requirements of environment commands.
  /// See [RetroEnvironmentData] for more information about type requirements.
  unsafe fn get_raw<T>(&self, cmd: impl Into<u32>) -> Option<T>
  where
    T: Default + RetroEnvironmentData;

  /// Directly invokes the underlying [retro_environment_t] in a "get" fashion.
  /// Returns `Some(T)` iff the command succeeds.
  ///
  /// `data` is transformed into a `T` value prior to calling the callback.
  ///
  /// # Safety
  /// See `libretro.h` for the requirements of environment commands.
  /// See [RetroEnvironmentData] for more information about type requirements.
  unsafe fn parameterized_get_raw<T>(&self, cmd: impl Into<u32>, data: impl Into<T>) -> Option<T>
  where
    T: RetroEnvironmentData;

  /// Directly invokes the underlying [retro_environment_t] in a "set" fashion.
  ///
  /// # Safety
  /// The environment command **must not** modify `data`.
  ///
  /// See `libretro.h` for the requirements of environment commands.
  /// See [RetroEnvironmentData] for more information about type requirements.
  unsafe fn set_raw(&mut self, cmd: impl Into<u32>, data: &impl RetroEnvironmentData) -> bool;

  /// Directly invokes the underlying [retro_environment_t] in a "set" fashion.
  /// Returns `Some(T)` iff the command succeeds.
  ///
  /// `data` is transformed into a `T` value prior to calling the callback.
  ///
  /// # Safety
  /// See `libretro.h` for the requirements of environment commands.
  /// See [RetroEnvironmentData] for more information about type requirements.
  unsafe fn parameterized_set_raw<T>(&mut self, cmd: impl Into<u32>, data: impl Into<T>) -> Option<T>
  where
    T: RetroEnvironmentData;

  /// Directly invokes the underlying [retro_environment_t] in a "command" fashion.
  ///
  /// # Safety
  /// See `libretro.h` for the requirements of environment commands.
  /// See [RetroEnvironmentData] for more information about type requirements.
  unsafe fn cmd_raw(&mut self, cmd: impl Into<u32>) -> bool;

  /// Gets an `Option<&c_char>` from [RetroEnvironment::get_raw] and converts it into a [CStr].
  ///
  /// # Safety
  /// The pointer returned by the command must be a valid argument to [CStr::from_ptr].
  ///
  /// See `libretro.h` for the requirements of environment commands.
  /// See [RetroEnvironmentData] for more information about type requirements.
  unsafe fn get_c_str_raw(&self, cmd: impl Into<u32>) -> OptionCStr {
    self
      .get_raw::<Option<&c_char>>(cmd)
      .flatten()
      .map(|x| CStr::from_ptr(x))
      .into()
  }

  /// Sets screen rotation of graphics.
  fn set_rotation(&mut self, rotation: ScreenRotation) -> bool {
    unsafe { self.set_raw(RETRO_ENVIRONMENT_SET_ROTATION, &rotation) }
  }

  #[cfg(deprecated)]
  /// Boolean value whether or not the implementation should use overscan,
  /// or crop away overscan.
  fn get_overscan(&self) -> bool {
    unsafe { self.get_bool(RETRO_ENVIRONMENT_GET_OVERSCAN) }
  }

  /// Boolean value indicating whether or not frontend supports frame duping.
  fn get_can_dupe(&self) -> bool {
    unsafe { self.get_raw(RETRO_ENVIRONMENT_GET_CAN_DUPE).unwrap_or(false) }
  }

  /// Sets a message to be displayed in implementation-specific manner for a
  /// certain amount of 'frames'. Should not be used for trivial messages,
  /// which should simply be logged via [RetroEnvironment::get_log_interface]
  /// (or as a fallback, stderr).
  fn set_message(&mut self, message: &RetroMessage) -> bool {
    unsafe { self.set_raw(RETRO_ENVIRONMENT_SET_MESSAGE, message) }
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

  /// Returns the "system" directory of the frontend. This directory can be used to store system
  /// specific content such as BIOSes, configuration data, etc. The returned value can be [None].
  /// If so, no such directory is defined, and it's up to the implementation to find a suitable
  /// directory.
  ///
  /// NOTE: Some cores used this folder also for "save" data such as memory cards, etc, for lack of
  /// a better place to put it. This is now discouraged, and if possible, cores should try to use
  /// the new GET_SAVE_DIRECTORY.
  fn get_system_directory(&self) -> OptionCStr {
    unsafe { self.get_c_str_raw(RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY) }
  }

  /// Queries the username associated with the frontend.
  fn get_username(&self) -> OptionCStr {
    unsafe { self.get_c_str_raw(RETRO_ENVIRONMENT_GET_USERNAME) }
  }

  /// Gets an interface for logging. This is useful for logging in a cross-platform way as certain
  /// platforms cannot use stderr for logging. It also allows the frontend to show logging
  /// information in a more suitable way. If this interface is not used, libretro cores should log
  /// to [std::io::Stderr] (via [eprintln], [StderrLogger] or [FallbackLogger]) as desired.
  fn get_log_interface(&self) -> Option<PlatformLogger> {
    unsafe { self.get_raw(RETRO_ENVIRONMENT_GET_LOG_INTERFACE) }
      .and_then(|cb: retro_log_callback| cb.log)
      .map(PlatformLogger::new)
  }
}

pub trait SetEnvironmentEnvironment: RetroEnvironment {
  fn set_support_no_game(&mut self, data: bool) -> bool {
    unsafe { self.set_raw(RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME, &data) }
  }
}
impl<T> SetEnvironmentEnvironment for T where T: RetroEnvironment {}

pub trait InitEnvironment: RetroEnvironment {}
impl<T> InitEnvironment for T where T: RetroEnvironment {}

pub trait SetPortDeviceEnvironment: RetroEnvironment {}
impl<T> SetPortDeviceEnvironment for T where T: RetroEnvironment {}

pub trait ResetEnvironment: RetroEnvironment {}
impl<T> ResetEnvironment for T where T: RetroEnvironment {}

pub trait RunEnvironment: RetroEnvironment {
  /// Requests that the frontend shut down. The frontend can refuse to do this, and return false.
  fn shutdown(&mut self) -> bool {
    unsafe { self.cmd_raw(RETRO_ENVIRONMENT_SHUTDOWN) }
  }

  fn set_geometry(&mut self, geometry: RetroGameGeometry) -> bool {
    unsafe { self.set_raw(RETRO_ENVIRONMENT_SET_GEOMETRY, &geometry) }
  }
}
impl<T> RunEnvironment for T where T: RetroEnvironment {}

pub trait SerializeSizeEnvironment: RetroEnvironment {}
impl<T> SerializeSizeEnvironment for T where T: RetroEnvironment {}

pub trait SerializeEnvironment: RetroEnvironment {}
impl<T> SerializeEnvironment for T where T: RetroEnvironment {}

pub trait UnserializeEnvironment: RetroEnvironment {}
impl<T> UnserializeEnvironment for T where T: RetroEnvironment {}

pub trait CheatResetEnvironment: RetroEnvironment {}
impl<T> CheatResetEnvironment for T where T: RetroEnvironment {}

pub trait CheatSetEnvironment: RetroEnvironment {}
impl<T> CheatSetEnvironment for T where T: RetroEnvironment {}

pub trait LoadGameEnvironment: RetroEnvironment {
  /// Gives a hint to the frontend how demanding this implementation is on a system. E.g. Reporting
  /// a level of 2 means this implementation should run decently on all frontends of level 2 and up.
  ///
  /// It can be used by the frontend to potentially warn about too demanding implementations.
  ///
  /// The levels are "floating".
  ///
  /// This function can be called on a per-game basis, as certain games an implementation can play
  /// might be particularly demanding.
  fn set_performance_level(&mut self, performance_level: impl Into<u32>) -> bool {
    unsafe { self.set_raw(RETRO_ENVIRONMENT_SET_PERFORMANCE_LEVEL, &performance_level.into()) }
  }

  /// Sets the internal pixel format used by the implementation.
  /// The default pixel format is RETRO_PIXEL_FORMAT_0RGB1555.
  /// This pixel format however, is deprecated (see enum retro_pixel_format).
  /// If the call returns false, the frontend does not support this pixel format.
  fn set_pixel_format(&mut self, format: RetroPixelFormat) -> bool {
    GetSystemAvInfoEnvironment::set_pixel_format(self, format)
  }
}
impl<T> LoadGameEnvironment for T where T: RetroEnvironment {}

pub trait GetSystemAvInfoEnvironment: RetroEnvironment {
  /// Sets the internal pixel format used by the implementation.
  /// The default pixel format is RETRO_PIXEL_FORMAT_0RGB1555.
  /// This pixel format however, is deprecated (see enum retro_pixel_format).
  /// If the call returns false, the frontend does not support this pixel format.
  fn set_pixel_format(&mut self, format: RetroPixelFormat) -> bool {
    unsafe { self.set_raw(RETRO_ENVIRONMENT_SET_PIXEL_FORMAT, &format) }
  }
}
impl<T> GetSystemAvInfoEnvironment for T where T: RetroEnvironment {}

pub trait GetRegionEnvironment: RetroEnvironment {}
impl<T> GetRegionEnvironment for T where T: RetroEnvironment {}

pub trait LoadGameSpecialEnvironment: RetroEnvironment {}
impl<T> LoadGameSpecialEnvironment for T where T: RetroEnvironment {}

pub trait UnloadGameEnvironment: RetroEnvironment {}
impl<T> UnloadGameEnvironment for T where T: RetroEnvironment {}

pub trait GetMemoryDataEnvironment: RetroEnvironment {}
impl<T> GetMemoryDataEnvironment for T where T: RetroEnvironment {}

pub trait GetMemorySizeEnvironment: RetroEnvironment {}
impl<T> GetMemorySizeEnvironment for T where T: RetroEnvironment {}

unsafe fn get_option_from_callback<T>(cb: &EnvironmentCallback, cmd: impl Into<u32>, mut data: T) -> Option<T> {
  if cb(cmd.into(), &mut data as *mut _ as *mut c_void) {
    Some(data)
  } else {
    None
  }
}

impl RetroEnvironment for EnvironmentCallback {
  unsafe fn get_raw<T>(&self, cmd: impl Into<u32>) -> Option<T>
  where
    T: Default + RetroEnvironmentData,
  {
    get_option_from_callback(self, cmd.into(), T::default())
  }

  unsafe fn parameterized_get_raw<T>(&self, cmd: impl Into<u32>, data: impl Into<T>) -> Option<T>
  where
    T: RetroEnvironmentData,
  {
    get_option_from_callback(self, cmd, data.into())
  }

  unsafe fn set_raw(&mut self, cmd: impl Into<u32>, data: &impl RetroEnvironmentData) -> bool {
    self(cmd.into(), data as *const _ as *mut c_void)
  }

  unsafe fn parameterized_set_raw<T>(&mut self, cmd: impl Into<u32>, data: impl Into<T>) -> Option<T>
  where
    T: RetroEnvironmentData,
  {
    get_option_from_callback(self, cmd, data.into())
  }

  unsafe fn cmd_raw(&mut self, cmd: impl Into<u32>) -> bool {
    self(cmd.into(), core::ptr::null_mut())
  }
}

impl RetroEnvironment for NullEnvironment {
  unsafe fn get_raw<T>(&self, _cmd: impl Into<u32>) -> Option<T>
  where
    T: Default + RetroEnvironmentData,
  {
    None
  }

  unsafe fn parameterized_get_raw<T>(&self, _cmd: impl Into<u32>, _data: impl Into<T>) -> Option<T>
  where
    T: RetroEnvironmentData,
  {
    None
  }

  unsafe fn set_raw(&mut self, _cmd: impl Into<u32>, _data: &impl RetroEnvironmentData) -> bool {
    false
  }

  unsafe fn parameterized_set_raw<T>(&mut self, _cmd: impl Into<u32>, _data: impl Into<T>) -> Option<T>
  where
    T: RetroEnvironmentData,
  {
    None
  }

  unsafe fn cmd_raw(&mut self, _cmd: impl Into<u32>) -> bool {
    false
  }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ScreenRotation {
  #[default]
  ZeroDegrees = 0,
  NinetyDegrees = 1,
  OneEightyDegrees = 2,
  TwoSeventyDegrees = 3,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RetroMessage(retro_message);

impl RetroMessage {
  pub fn new<'a>(msg: impl Into<&'a CUtf8>, frames: u32) -> Self {
    Self(retro_message {
      msg: msg.into().as_ptr(),
      frames,
    })
  }

  pub fn msg(&self) -> &CUtf8 {
    unsafe { CUtf8::from_c_str_unchecked(CStr::from_ptr(self.0.msg)) }
  }

  pub fn frames(&self) -> u32 {
    self.0.frames
  }
}
