mod callback_environment;
mod convert;
mod newtypes;
mod null_environment;

use crate::prelude::*;
use crate::retro::*;
pub use callback_environment::*;
pub use convert::*;
pub use newtypes::*;
pub use null_environment::*;

pub type Result<T> = core::result::Result<T, CommandError>;

/// Exposes the [`retro_environment_t`] callback in an idiomatic fashion.
/// Each of the `RETRO_ENVIRONMENT_*` keys will eventually have a corresponding method here.
///
/// Until that is accomplished, the keys are available in [`libretro_rs::ffi`] and can be used
/// manually with the various `*_raw` methods.
pub trait Environment: Sized {
  unsafe fn get<C, D, R>(&self, cmd: C) -> Result<R>
  where
    C: Into<u32>,
    D: CommandData + Default,
    R: CommandOutput<Source = D>,
  {
    self.get_raw::<C, D>(cmd).map(|data| R::unsafe_from(data))
  }

  /// Directly invokes the underlying [retro_environment_t] in a "get" fashion.
  ///
  /// # Safety
  /// See `libretro.h` for the requirements of environment commands.
  /// See [CommandData] for more information about type requirements.
  unsafe fn get_raw<C, D>(&self, cmd: C) -> Result<D>
  where
    C: Into<u32>,
    D: Default + CommandData,
  {
    let mut data = D::default();
    self.parameterized_get_raw(cmd, &mut data).map(|_| data)
  }

  unsafe fn parameterized_get<C, D, R>(&self, cmd: C, data: D) -> Result<R>
  where
    C: Into<u32>,
    D: Into<R::Source>,
    R: CommandOutput,
  {
    let mut data: R::Source = data.into();
    self.parameterized_get_raw(cmd, &mut data).map(|_| R::unsafe_from(data))
  }

  /// Directly invokes the underlying [retro_environment_t] in a "get" fashion.
  ///
  /// `data` is transformed into a `T` value prior to calling the callback.
  ///
  /// # Safety
  /// See `libretro.h` for the requirements of environment commands.
  /// See [CommandData] for more information about type requirements.
  unsafe fn parameterized_get_raw<C, D>(&self, cmd: C, data: &mut D) -> Result<()>
  where
    C: Into<u32>,
    D: CommandData;

  /// Directly invokes the underlying [retro_environment_t] in a "set" fashion.
  ///
  /// # Safety
  /// The environment command **must not** modify `data`.
  ///
  /// See `libretro.h` for the requirements of environment commands.
  /// See [CommandData] for more information about type requirements.
  unsafe fn set_raw<C, D>(&mut self, cmd: C, data: &D) -> Result<()>
  where
    C: Into<u32>,
    D: CommandData;

  /// Directly invokes the underlying [retro_environment_t] in a "command" fashion.
  /// Equivalent to [Environment::set_raw] with `&()`.
  ///
  /// # Safety
  /// See `libretro.h` for the requirements of environment commands.
  /// See [CommandData] for more information about type requirements.
  unsafe fn cmd(&mut self, cmd: impl Into<u32>) -> Result<()> {
    self.set_raw(cmd, &())
  }

  unsafe fn parameterized_cmd<C, D, R>(&mut self, cmd: C, data: D) -> Result<R>
  where
    C: Into<u32>,
    D: Into<R::Source>,
    R: CommandOutput,
  {
    let mut data = data.into();
    self.parameterized_cmd_raw(cmd, &mut data).map(|_| R::unsafe_from(data))
  }

  /// Directly invokes the underlying [retro_environment_t] in a "command" fashion.
  /// Returns `Some(T)` iff the command succeeds.
  ///
  /// `data` is transformed into a `T` value prior to calling the callback.
  ///
  /// # Safety
  /// See `libretro.h` for the requirements of environment commands.
  /// See [CommandData] for more information about type requirements.
  unsafe fn parameterized_cmd_raw<C, D>(&mut self, cmd: C, data: &mut D) -> Result<()>
  where
    C: Into<u32>,
    D: CommandData,
  {
    self.set_raw(cmd, data)
  }

  /// Sets screen rotation of graphics.
  fn set_rotation(&mut self, rotation: ScreenRotation) -> Result<()> {
    unsafe { self.set_raw(RETRO_ENVIRONMENT_SET_ROTATION, &rotation) }
  }

  #[cfg(deprecated)]
  /// Boolean value whether or not the implementation should use overscan,
  /// or crop away overscan.
  fn get_overscan(&self) -> Result<bool> {
    unsafe { self.get_raw(RETRO_ENVIRONMENT_GET_OVERSCAN) }
  }

  /// Boolean value indicating whether or not frontend supports frame duping.
  fn get_can_dupe(&self) -> Result<bool> {
    unsafe { self.get_raw(RETRO_ENVIRONMENT_GET_CAN_DUPE) }
  }

  /// Sets a message to be displayed in implementation-specific manner for a
  /// certain amount of 'frames'. Should not be used for trivial messages,
  /// which should simply be logged via [Environment::get_log_interface]
  /// (or as a fallback, stderr).
  fn set_message(&mut self, message: impl Into<Message>) -> Result<()> {
    let message: retro_message = message.into().into();
    unsafe { self.set_raw(RETRO_ENVIRONMENT_SET_MESSAGE, &message) }
  }

  /// Queries the path where the current libretro core resides.
  fn get_libretro_path(&self) -> Result<Option<&CStr>> {
    unsafe { self.get(RETRO_ENVIRONMENT_GET_LIBRETRO_PATH) }
  }

  /// Queries the path of the "core assets" directory.
  fn get_core_assets_directory(&self) -> Result<Option<&CStr>> {
    unsafe { self.get(RETRO_ENVIRONMENT_GET_CORE_ASSETS_DIRECTORY) }
  }

  /// Queries the path of the save directory.
  fn get_save_directory(&self) -> Result<Option<&CStr>> {
    unsafe { self.get(RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY) }
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
    unsafe { self.get(RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY) }
  }

  fn get_variable(&self, key: &impl AsRef<CStr>) -> Result<RetroVariable> {
    unsafe { self.parameterized_get(RETRO_ENVIRONMENT_GET_VARIABLE, key.as_ref()) }
  }

  /// Queries the username associated with the frontend.
  fn get_username(&self) -> Result<Option<&CStr>> {
    unsafe { self.get(RETRO_ENVIRONMENT_GET_USERNAME) }
  }

  /// Gets an interface for logging. This is useful for logging in a cross-platform way as certain
  /// platforms cannot use stderr for logging. It also allows the frontend to show logging
  /// information in a more suitable way. If this interface is not used, libretro cores should log
  /// to [std::io::Stderr] (via [eprintln], [StderrLogger] or [FallbackLogger]) as desired.
  fn get_log_interface(&self) -> Result<PlatformLogger> {
    unsafe { self.get(RETRO_ENVIRONMENT_GET_LOG_INTERFACE) }
  }
}

pub trait SetEnvironmentEnvironment: Environment {
  fn set_support_no_game(&mut self, data: bool) -> Result<()> {
    unsafe { self.set_raw(RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME, &data) }
  }
}
impl<T> SetEnvironmentEnvironment for T where T: Environment {}

pub trait InitEnvironment: Environment {}
impl<T> InitEnvironment for T where T: Environment {}

pub trait SetPortDeviceEnvironment: Environment {}
impl<T> SetPortDeviceEnvironment for T where T: Environment {}

pub trait ResetEnvironment: Environment {}
impl<T> ResetEnvironment for T where T: Environment {}

pub trait RunEnvironment: Environment {
  /// Requests that the frontend shut down. The frontend can refuse to do this, and return false.
  fn shutdown(&mut self) -> Result<()> {
    unsafe { self.cmd(RETRO_ENVIRONMENT_SHUTDOWN) }
  }

  fn set_geometry(&mut self, geometry: impl Into<GameGeometry>) -> Result<()> {
    let geometry: retro_game_geometry = geometry.into().into();
    unsafe { self.set_raw(RETRO_ENVIRONMENT_SET_GEOMETRY, &geometry) }
  }
}
impl<T> RunEnvironment for T where T: Environment {}

pub trait SerializeSizeEnvironment: Environment {}
impl<T> SerializeSizeEnvironment for T where T: Environment {}

pub trait SerializeEnvironment: Environment {}
impl<T> SerializeEnvironment for T where T: Environment {}

pub trait UnserializeEnvironment: Environment {}
impl<T> UnserializeEnvironment for T where T: Environment {}

pub trait CheatResetEnvironment: Environment {}
impl<T> CheatResetEnvironment for T where T: Environment {}

pub trait CheatSetEnvironment: Environment {}
impl<T> CheatSetEnvironment for T where T: Environment {}

pub trait LoadGameEnvironment: Environment {
  /// Gives a hint to the frontend how demanding this implementation is on a system. E.g. Reporting
  /// a level of 2 means this implementation should run decently on all frontends of level 2 and up.
  ///
  /// It can be used by the frontend to potentially warn about too demanding implementations.
  ///
  /// The levels are "floating".
  ///
  /// This function can be called on a per-game basis, as certain games an implementation can play
  /// might be particularly demanding.
  fn set_performance_level(&mut self, performance_level: impl Into<u32>) -> Result<()> {
    unsafe { self.set_raw(RETRO_ENVIRONMENT_SET_PERFORMANCE_LEVEL, &performance_level.into()) }
  }

  /// Sets the internal pixel format used by the implementation.
  /// The default pixel format is RETRO_PIXEL_FORMAT_0RGB1555.
  /// This pixel format however, is deprecated (see enum retro_pixel_format).
  /// If the call returns false, the frontend does not support this pixel format.
  fn set_pixel_format(&mut self, format: PixelFormat) -> Result<()> {
    GetSystemAvInfoEnvironment::set_pixel_format(self, format)
  }
}
impl<T> LoadGameEnvironment for T where T: Environment {}

pub trait GetSystemAvInfoEnvironment: Environment {
  /// Sets the internal pixel format used by the implementation.
  /// The default pixel format is RETRO_PIXEL_FORMAT_0RGB1555.
  /// This pixel format however, is deprecated (see enum retro_pixel_format).
  /// If the call returns false, the frontend does not support this pixel format.
  fn set_pixel_format(&mut self, format: PixelFormat) -> Result<()> {
    unsafe { self.set_raw(RETRO_ENVIRONMENT_SET_PIXEL_FORMAT, &format) }
  }
}
impl<T> GetSystemAvInfoEnvironment for T where T: Environment {}

pub trait GetRegionEnvironment: Environment {}
impl<T> GetRegionEnvironment for T where T: Environment {}

pub trait LoadGameSpecialEnvironment: Environment {}
impl<T> LoadGameSpecialEnvironment for T where T: Environment {}

pub trait UnloadGameEnvironment: Environment {}
impl<T> UnloadGameEnvironment for T where T: Environment {}

pub trait GetMemoryDataEnvironment: Environment {}
impl<T> GetMemoryDataEnvironment for T where T: Environment {}

pub trait GetMemorySizeEnvironment: Environment {}
impl<T> GetMemorySizeEnvironment for T where T: Environment {}
