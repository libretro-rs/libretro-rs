pub use crate::convert::*;
use crate::ffi::*;
use crate::retro::pixel::{Format, ORGB1555, RGB565, XRGB8888};
use crate::retro::*;
use libretro_rs_ffi::retro_pixel_format::{
  RETRO_PIXEL_FORMAT_0RGB1555, RETRO_PIXEL_FORMAT_RGB565, RETRO_PIXEL_FORMAT_XRGB8888,
};
use std::marker::PhantomData;

pub type Result<T> = core::result::Result<T, CommandError>;

/// Exposes the [`retro_environment_t`] callback in an idiomatic fashion.
/// Each of the `RETRO_ENVIRONMENT_*` keys will eventually have a corresponding method here.
///
/// Until that is accomplished, the keys are available in [`libretro_rs::ffi`] and can be used
/// manually with the various `*_raw` methods.
pub trait Environment: Sized {
  fn get_ptr(&self) -> non_null_retro_environment_t;

  /// Directly invokes the underlying [retro_environment_t] in a "get" fashion.
  ///
  /// # Safety
  /// See `libretro.h` for the requirements of environment commands.
  /// See [CommandData] for more information about type requirements.
  unsafe fn get<C, R>(&self, cmd: C) -> Result<R>
  where
    C: Into<c_uint>,
    R: Default + CommandData,
  {
    self.get_with(cmd, R::default())
  }

  /// Directly invokes the underlying [retro_environment_t] in a "get" fashion.
  ///
  /// `data` is transformed into a `T` value prior to calling the callback.
  ///
  /// # Safety
  /// See `libretro.h` for the requirements of environment commands.
  /// See [CommandData] for more information about type requirements.
  unsafe fn get_with<C, D, R>(&self, cmd: C, data: D) -> Result<R>
  where
    C: Into<c_uint>,
    D: Into<R>,
    R: CommandData,
  {
    let mut data = data.into();
    with_mut(self.get_ptr(), cmd.into(), &mut data).map(|_| data)
  }

  unsafe fn set<C, D>(&mut self, cmd: C, data: &D) -> Result<()>
  where
    C: Into<c_uint>,
    D: CommandData,
  {
    with_ref(self.get_ptr(), cmd.into(), data)
  }

  unsafe fn cmd<C, D, R>(&mut self, cmd: C, data: D) -> Result<R>
  where
    C: Into<c_uint>,
    D: Into<R>,
    R: CommandData,
  {
    let mut data = data.into();
    with_mut(self.get_ptr(), cmd.into(), &mut data).map(|_| data)
  }

  /// Sets screen rotation of graphics.
  fn set_rotation(&mut self, rotation: ScreenRotation) -> Result<()> {
    unsafe { self.set(RETRO_ENVIRONMENT_SET_ROTATION, &(rotation as c_int)) }
  }

  #[cfg(deprecated)]
  /// Boolean value whether or not the implementation should use overscan,
  /// or crop away overscan.
  fn get_overscan(&self) -> Result<bool> {
    unsafe { self.get(RETRO_ENVIRONMENT_GET_OVERSCAN) }
  }

  /// Boolean value indicating whether or not frontend supports frame duping.
  fn get_can_dupe(&self) -> Result<bool> {
    unsafe { self.get(RETRO_ENVIRONMENT_GET_CAN_DUPE) }
  }

  /// Sets a message to be displayed in implementation-specific manner for a
  /// certain amount of 'frames'. Should not be used for trivial messages,
  /// which should simply be logged via [Environment::get_log_interface]
  /// (or as a fallback, stderr).
  fn set_message(&mut self, message: &Message) -> Result<()> {
    unsafe { self.set(RETRO_ENVIRONMENT_SET_MESSAGE, message) }
  }

  /// Queries the path where the current libretro core resides.
  fn get_libretro_path(&self) -> Result<Option<&CStr>> {
    unsafe { self.get(RETRO_ENVIRONMENT_GET_LIBRETRO_PATH).unsafe_into() }
  }

  /// Queries the path of the "core assets" directory.
  fn get_core_assets_directory(&self) -> Result<Option<&CStr>> {
    unsafe {
      self
        .get(RETRO_ENVIRONMENT_GET_CORE_ASSETS_DIRECTORY)
        .unsafe_into()
    }
  }

  /// Queries the path of the save directory.
  fn get_save_directory(&self) -> Result<Option<&CStr>> {
    unsafe { self.get(RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY).unsafe_into() }
  }

  /// Returns the "system" directory of the frontend. This directory can be used to store system
  /// specific content such as BIOSes, configuration data, etc. The returned value can be [None].
  /// If so, no such directory is defined, and it's up to the implementation to find a suitable
  /// directory.
  ///
  /// NOTE: Some cores used this folder also for "save" data such as memory cards, etc, for lack of
  /// a better place to put it. This is now discouraged, and if possible, cores should try to use
  /// the new GET_SAVE_DIRECTORY.
  fn get_system_directory(&self) -> Result<Option<&CStr>> {
    unsafe {
      self
        .get(RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY)
        .unsafe_into()
    }
  }

  fn get_variable(&self, key: &impl AsRef<CStr>) -> Result<Option<&CStr>> {
    let variable = retro_variable {
      key: key.as_ref().as_ptr(),
      value: core::ptr::null(),
    };
    unsafe {
      self
        .get_with(RETRO_ENVIRONMENT_GET_VARIABLE, variable)
        .map(|var: retro_variable| var.value.as_ref().map(|ptr| CStr::from_ptr(ptr)))
    }
  }

  /// Queries the username associated with the frontend.
  fn get_username(&self) -> Result<Option<&CStr>> {
    unsafe { self.get(RETRO_ENVIRONMENT_GET_USERNAME).unsafe_into() }
  }

  /// Gets an interface for logging. This is useful for logging in a cross-platform way as certain
  /// platforms cannot use stderr for logging. It also allows the frontend to show logging
  /// information in a more suitable way. If this interface is not used, libretro cores should log
  /// to [std::io::Stderr] (via [eprintln], [StderrLogger] or [FallbackLogger]) as desired.
  fn get_log_interface(&self) -> Result<PlatformLogger> {
    unsafe { self.get(RETRO_ENVIRONMENT_GET_LOG_INTERFACE).unsafe_into() }
  }
}

impl Environment for non_null_retro_environment_t {
  fn get_ptr(&self) -> non_null_retro_environment_t {
    *self
  }
}

pub trait SetEnvironment: Environment {
  fn set_support_no_game(&mut self, data: bool) -> Result<()> {
    unsafe { self.set(RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME, &data) }
  }
}
impl<T: Environment> SetEnvironment for T {}

pub trait Init: Environment {}
impl<T: Environment> Init for T {}

pub trait SetPortDevice: Environment {}
impl<T: Environment> SetPortDevice for T {}

pub trait Reset: Environment {}
impl<T: Environment> Reset for T {}

pub trait Run: Environment {
  /// Requests that the frontend shut down. The frontend can refuse to do this, and return false.
  fn shutdown(&mut self) -> Result<()> {
    unsafe { self.cmd(RETRO_ENVIRONMENT_SHUTDOWN, ()) }
  }

  fn set_geometry(&mut self, geometry: &GameGeometry) -> Result<()> {
    unsafe { self.set(RETRO_ENVIRONMENT_SET_GEOMETRY, geometry) }
  }
}
impl<T: Environment> Run for T {}

pub trait SerializeSize: Environment {}
impl<T: Environment> SerializeSize for T {}

pub trait Serialize: Environment {}
impl<T: Environment> Serialize for T {}

pub trait Unserialize: Environment {}
impl<T: Environment> Unserialize for T {}

pub trait CheatReset: Environment {}
impl<T: Environment> CheatReset for T {}

pub trait CheatSet: Environment {}
impl<T: Environment> CheatSet for T {}

pub trait LoadGame: Environment {
  /// Gives a hint to the frontend how demanding this implementation is on a system. E.g. Reporting
  /// a level of 2 means this implementation should run decently on all frontends of level 2 and up.
  ///
  /// It can be used by the frontend to potentially warn about too demanding implementations.
  ///
  /// The levels are "floating".
  ///
  /// This function can be called on a per-game basis, as certain games an implementation can play
  /// might be particularly demanding.
  fn set_performance_level(&mut self, performance_level: impl Into<c_uint>) -> Result<()> {
    unsafe {
      self.set(
        RETRO_ENVIRONMENT_SET_PERFORMANCE_LEVEL,
        &performance_level.into(),
      )
    }
  }

  fn set_pixel_format_0rgb1555<F>(
    &mut self,
    current_format: Format<F>,
  ) -> core::result::Result<Format<ORGB1555>, Format<F>> {
    GetAvInfo::set_pixel_format_0rgb1555(self, current_format)
  }

  fn set_pixel_format_xrgb8888<F>(
    &mut self,
    current_format: Format<F>,
  ) -> core::result::Result<Format<XRGB8888>, Format<F>> {
    GetAvInfo::set_pixel_format_xrgb8888(self, current_format)
  }

  fn set_pixel_format_rgb565<F>(
    &mut self,
    current_format: Format<F>,
  ) -> core::result::Result<Format<RGB565>, Format<F>> {
    GetAvInfo::set_pixel_format_rgb565(self, current_format)
  }

  fn set_hw_render_none(&mut self) -> Result<()>;

  fn set_hw_render_gl(&mut self, options: GLOptions) -> Result<GLRenderEnabled>;
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Origin {
  #[default]
  TopLeft,
  BottomRight,
}

pub trait GetAvInfo: Environment {
  #[allow(unused_variables)]
  fn set_pixel_format_0rgb1555<F>(
    &mut self,
    current_format: Format<F>,
  ) -> core::result::Result<Format<ORGB1555>, Format<F>> {
    unsafe {
      self
        .set(
          RETRO_ENVIRONMENT_SET_PIXEL_FORMAT,
          &RETRO_PIXEL_FORMAT_0RGB1555,
        )
        .map(|_| Format(PhantomData))
        .map_err(|_| current_format)
    }
  }

  #[allow(unused_variables)]
  fn set_pixel_format_xrgb8888<F>(
    &mut self,
    current_format: Format<F>,
  ) -> core::result::Result<Format<XRGB8888>, Format<F>> {
    unsafe {
      self
        .set(
          RETRO_ENVIRONMENT_SET_PIXEL_FORMAT,
          &RETRO_PIXEL_FORMAT_XRGB8888,
        )
        .map(|_| Format(PhantomData))
        .map_err(|_| current_format)
    }
  }

  #[allow(unused_variables)]
  fn set_pixel_format_rgb565<F>(
    &mut self,
    current_format: Format<F>,
  ) -> core::result::Result<Format<RGB565>, Format<F>> {
    unsafe {
      self
        .set(
          RETRO_ENVIRONMENT_SET_PIXEL_FORMAT,
          &RETRO_PIXEL_FORMAT_RGB565,
        )
        .map(|_| Format(PhantomData))
        .map_err(|_| current_format)
    }
  }
}
impl<T: Environment> GetAvInfo for T {}

pub trait GetRegion: Environment {}
impl<T: Environment> GetRegion for T {}

pub trait LoadGameSpecial: Environment {}
impl<T: Environment> LoadGameSpecial for T {}

pub trait UnloadGame: Environment {}
impl<T: Environment> UnloadGame for T {}

pub trait GetMemoryData: Environment {}
impl<T: Environment> GetMemoryData for T {}

pub trait GetMemorySize: Environment {}
impl<T: Environment> GetMemorySize for T {}

pub trait Deinit: Environment {}
impl<T: Environment> Deinit for T {}

unsafe fn with_ref(
  cb: non_null_retro_environment_t,
  cmd: c_uint,
  data: &impl CommandData,
) -> Result<()> {
  if cb(cmd, data as *const _ as *mut c_void) {
    Ok(())
  } else {
    Err(CommandError::new())
  }
}

unsafe fn with_mut(
  cb: non_null_retro_environment_t,
  cmd: c_uint,
  data: &mut impl CommandData,
) -> Result<()> {
  if cb(cmd, data as *mut _ as *mut c_void) {
    Ok(())
  } else {
    Err(CommandError::new())
  }
}

pub extern "C" fn null_environment(_cmd: c_uint, _data: *mut c_void) -> bool {
  false
}

/// Marker trait for types that are valid arguments to the environment callback.
///
/// Any type implementing this trait must be FFI-safe. Structs should be `#[repr(C)]` or a
/// `#[repr(transparent)]` newtype. Numeric enums should have the appropriate primitive
/// representation, which is typically either `#[repr(core::ffi::c_uint)]` for
/// `const unsigned` arguments or `#[repr(core::ffi::c_int)]` for `const enum` arguments.
///
/// Care must still be taken when calling any of the generic unsafe `[RetroEnvironment]` methods to
/// ensure the type used is appropriate for the environment command, as specified in `libretro.h`.
pub trait CommandData {}
impl CommandData for () {}
impl CommandData for bool {}
impl CommandData for c_int {}
impl CommandData for c_uint {}
impl CommandData for Option<&c_char> {}
impl CommandData for Option<&c_void> {}
impl CommandData for retro_hw_render_callback {}
impl CommandData for retro_game_geometry {}
impl CommandData for GameGeometry {}
impl CommandData for retro_log_callback {}
impl CommandData for retro_message {}
impl CommandData for Message {}
impl CommandData for retro_pixel_format {}
impl CommandData for retro_system_av_info {}
impl CommandData for SystemAVInfo {}
impl CommandData for retro_variable {}
