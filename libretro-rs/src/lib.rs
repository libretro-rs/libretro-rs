#![allow(unused_variables)]

use std::ffi::CStr;

pub use libc;

pub mod core_macro;
pub mod sys;

use libc::c_void;
use sys::*;

pub trait RetroCore {
  /// Called when a new instance of the core is needed. The `env` parameter can be used to set-up and/or query values
  /// required for the core to function properly.
  fn new(env: &RetroEnvironment) -> Self;

  /// Called to get information about the core. This information can then be displayed in a frontend, or used to
  /// construct core-specific paths.
  fn get_system_info(info: &mut retro_system_info);

  /// Called to get audio/video parameters. This is guaranteed to be called _after_ `retro_load_game`, so that a core
  /// can decide how to emulate the game.
  fn get_system_av_info(&self, info: &mut retro_system_av_info);

  /// Called to associate a particular device with a particular port. A core is allowed to ignore this request.
  fn set_controller_port_device(&mut self, port: u32, device: RetroDevice);

  /// Called when a player resets their game.
  fn reset(&mut self);

  /// Called continuously once the core is initialized and a game is loaded. The core is expected to advance emulation
  /// by a single frame before returning.
  fn run(&mut self);

  /// Called to determine the size of the save state buffer. This is only ever called once per run, and the core must
  /// not exceed the size returned here for subsequent saves.
  fn serialize_size(&self) -> usize {
    0
  }

  /// Allows a core to save its internal state into the specified buffer. The buffer is guaranteed to be at least `size`
  /// bytes, where `size` is the value returned from `serialize_size`.
  fn serialize(&self, data: *mut (), size: usize) -> bool {
    false
  }

  /// Allows a core to load its internal state from the specified buffer. The buffer is guaranteed to be at least `size`
  /// bytes, where `size` is the value returned from `serialize_size`.
  fn unserialize(&mut self, data: *const (), size: usize) -> bool {
    false
  }

  fn cheat_reset(&mut self) {}

  fn cheat_set(&mut self, index: u32, enabled: bool, code: *const libc::c_char) {}

  fn load_game(&mut self, game: RetroGame) -> bool;

  fn load_game_special(&mut self, game_type: u32, info: RetroGame, num_info: usize) -> bool {
    false
  }

  fn unload_game(&mut self) {}

  fn get_region(&self) -> RetroRegion {
    RetroRegion::NTSC
  }

  fn get_memory_data(&mut self, id: u32) -> *mut () {
    std::ptr::null_mut()
  }

  fn get_memory_size(&self, id: u32) -> usize {
    0
  }
}

#[derive(Debug)]
pub enum RetroDevice {
  None = 0,
  Joypad = 1,
  Mouse = 2,
  Keyboard = 3,
  LightGun = 4,
  Analog = 5,
  Pointer = 6,
}

impl From<u32> for RetroDevice {
  fn from(val: u32) -> Self {
    match val {
      0 => Self::None,
      1 => Self::Joypad,
      2 => Self::Mouse,
      3 => Self::Keyboard,
      4 => Self::LightGun,
      5 => Self::Analog,
      6 => Self::Pointer,
      _ => panic!("unrecognized device type. type={}", val),
    }
  }
}

pub enum RetroJoypadButton {
  B = 0,
  Y = 1,
  Select = 2,
  Start = 3,
  Up = 4,
  Down = 5,
  Left = 6,
  Right = 7,
  A = 8,
  X = 9,
  L = 10,
  R = 11,
  L2 = 12,
  R2 = 13,
  L3 = 14,
  R3 = 15,
}

/// Exposes the `retro_environment_t` callback in an idiomatic fashion. Each of the `RETRO_ENVIRONMENT_*` keys will
/// eventually have a corresponding method here.
///
/// Until that is accomplished, the keys are available in `libretro_rs::sys` and can be used manually with the `get_raw`,
/// `get`, `get_str` and `set_raw` functions.
pub struct RetroEnvironment(retro_environment_t);

impl RetroEnvironment {
  pub fn new(cb: retro_environment_t) -> RetroEnvironment {
    RetroEnvironment(cb)
  }

  /* Commands */

  /// Requests that the frontend shut down. The frontend can refuse to do this, and return false.
  pub fn shutdown(&self) -> bool {
    unsafe {
      (self.0.unwrap())(RETRO_ENVIRONMENT_SHUTDOWN, std::ptr::null_mut())
    }
  }

  /* Queries */

  /// Queries the path where the current libretro core resides.
  pub fn get_libretro_path(&self) -> Option<&str> {
    self.get_str(RETRO_ENVIRONMENT_GET_LIBRETRO_PATH)
  }

  /// Queries the path of the "core assets" directory.
  pub fn get_core_assets_directory(&self) -> Option<&str> {
    self.get_str(RETRO_ENVIRONMENT_GET_CORE_ASSETS_DIRECTORY)
  }

  /// Queries the path of the save directory.
  pub fn get_save_directory(&self) -> Option<&str> {
    self.get_str(RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY)
  }

  /// Queries the path of the system directory.
  pub fn get_system_directory(&self) -> Option<&str> {
    self.get_str(RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY)
  }

  /// Queries the username associated with the frontend.
  pub fn get_username(&self) -> Option<&str> {
    self.get_str(RETRO_ENVIRONMENT_GET_USERNAME)
  }

  /// Queries a string slice from the environment. A null pointer (`*const c_char`) is interpreted as `None`.
  pub fn get_str<'a>(&'a self, key: u32) -> Option<&'a str> {
    unsafe {
      let s = self.get(key)?;
      CStr::from_ptr(s).to_str().ok()
    }
  }

  /// Queries a struct from the environment. A null pointer (`*const T`) is interpreted as `None`.
  pub unsafe fn get<T>(&self, key: u32) -> Option<*const T> {
    let mut val: *const T = std::ptr::null();
    if self.get_raw(key, &mut val) && !val.is_null() {
      Some(val)
    } else {
      None
    }
  }

  /// Simply invokes the underlying `retro_environment_t` in a "get" fashion.
  pub unsafe fn get_raw<T>(&self, key: u32, output: *mut *const T) -> bool {
    self.0.unwrap()(key, output as *mut c_void)
  }

  /// Simply invokes the underlying `retro_environment_t` in a "set" fashion.
  pub unsafe fn set_raw<T>(&self, key: u32, input: *const T) -> bool {
    self.0.unwrap()(key, input as *mut c_void)
  }
}

/// Represents the possible ways that a frontend can pass game information to a core.
pub enum RetroGame<'a> {
  /// Passed if a core supports "no game" mode and no game was selected.
  None,
  /// Passed if a core doesn't need paths, and a game was selected.
  /// The slice contains the entire contents of the game.
  Data(&'a [u8]),
  /// Passed if a core does need paths, and a game was selected.
  /// The slice contains the entire absolute path of the game.
  Path(&'a str),
}

impl<'a> From<&retro_game_info> for RetroGame<'a> {
  fn from(game: &retro_game_info) -> RetroGame<'a> {
    if game.path.is_null() && game.data.is_null() {
      return RetroGame::None;
    }

    if !game.path.is_null() {
      unsafe {
        let path = game.path;
        let path = CStr::from_ptr(path).to_str().unwrap();
        return RetroGame::Path(path);
      }
    }

    if !game.data.is_null() {
      unsafe {
        let data = game.data;
        let size = game.size;
        let data = std::slice::from_raw_parts(data as *const u8, size);
        return RetroGame::Data(data);
      }
    }

    unreachable!("`game_info` has a `path` and a `data` field.")
  }
}

/// Represents the set of regions supported by `libretro`.
pub enum RetroRegion {
  /// Most known for its 30 frames/second (60 fields/second) video system.
  NTSC = 0,
  /// Most known for its 25 frames/second (50 fields/second) video system.
  PAL = 1,
}

impl Into<u32> for RetroRegion {
  fn into(self) -> u32 {
    match self {
      Self::NTSC => 0,
      Self::PAL => 1,
    }
  }
}

/// This is the glue layer between a `RetroCore` implementation, and the `libretro` API.
pub struct RetroInstance<T: RetroCore> {
  pub core: Option<T>,
  pub audio_sample: retro_audio_sample_t,
  pub audio_sample_batch: retro_audio_sample_batch_t,
  pub environment: retro_environment_t,
  pub input_poll: retro_input_poll_t,
  pub input_state: retro_input_state_t,
  pub video_refresh: retro_video_refresh_t,
}

impl<T: RetroCore> RetroInstance<T> {
  /// Invoked by a `libretro` frontend, with the `retro_get_system_info` API call.
  pub fn on_get_system_info(&self, info: &mut retro_system_info) {
    T::get_system_info(info)
  }

  /// Invoked by a `libretro` frontend, with the `retro_get_system_av_info` API call.
  pub fn on_get_system_av_info(&self, info: &mut retro_system_av_info) {
    self.core_ref(|core| core.get_system_av_info(info))
  }

  /// Invoked by a `libretro` frontend, with the `retro_init` API call.
  pub fn on_init(&mut self) {
    let env = RetroEnvironment::new(self.environment);
    self.core = Some(T::new(&env))
  }

  /// Invoked by a `libretro` frontend, with the `retro_deinit` API call.
  pub fn on_deinit(&mut self) {
    self.core = None;
    self.audio_sample = None;
    self.audio_sample_batch = None;
    self.environment = None;
    self.input_poll = None;
    self.input_state = None;
    self.video_refresh = None;
  }

  /// Invoked by a `libretro` frontend, with the `retro_set_environment` API call.
  pub fn on_set_environment(&mut self, cb: retro_environment_t) {
    self.environment = cb;
  }

  /// Invoked by a `libretro` frontend, with the `retro_set_audio_sample` API call.
  pub fn on_set_audio_sample(&mut self, cb: retro_audio_sample_t) {
    self.audio_sample = cb;
  }

  /// Invoked by a `libretro` frontend, with the `retro_set_audio_sample_batch` API call.
  pub fn on_set_audio_sample_batch(&mut self, cb: retro_audio_sample_batch_t) {
    self.audio_sample_batch = cb;
  }

  /// Invoked by a `libretro` frontend, with the `retro_set_input_poll` API call.
  pub fn on_set_input_poll(&mut self, cb: retro_input_poll_t) {
    self.input_poll = cb;
  }

  /// Invoked by a `libretro` frontend, with the `retro_set_input_state` API call.
  pub fn on_set_input_state(&mut self, cb: retro_input_state_t) {
    self.input_state = cb;
  }

  /// Invoked by a `libretro` frontend, with the `retro_set_video_refresh` API call.
  pub fn on_set_video_refresh(&mut self, cb: retro_video_refresh_t) {
    self.video_refresh = cb;
  }

  /// Invoked by a `libretro` frontend, with the `retro_set_controller_port_device` API call.
  pub fn on_set_controller_port_device(&mut self, port: libc::c_uint, device: libc::c_uint) {
    self.core_mut(|core| core.set_controller_port_device(port, device.into()))
  }

  /// Invoked by a `libretro` frontend, with the `retro_reset` API call.
  pub fn on_reset(&mut self) {
    self.core_mut(|core| core.reset())
  }

  /// Invoked by a `libretro` frontend, with the `retro_run` API call.
  pub fn on_run(&mut self) {
    self.core_mut(|core| core.run())
  }

  /// Invoked by a `libretro` frontend, with the `retro_serialize_size` API call.
  pub fn on_serialize_size(&self) -> libc::size_t {
    self.core_ref(|core| core.serialize_size())
  }

  /// Invoked by a `libretro` frontend, with the `retro_serialize` API call.
  pub fn on_serialize(&self, data: *mut (), size: libc::size_t) -> bool {
    self.core_ref(|core| core.serialize(data, size))
  }

  /// Invoked by a `libretro` frontend, with the `retro_unserialize` API call.
  pub fn on_unserialize(&mut self, data: *const (), size: libc::size_t) -> bool {
    self.core_mut(|core| core.unserialize(data, size))
  }

  /// Invoked by a `libretro` frontend, with the `retro_cheat_reset` API call.
  pub fn on_cheat_reset(&mut self) {
    self.core_mut(|core| core.cheat_reset())
  }

  /// Invoked by a `libretro` frontend, with the `retro_cheat_set` API call.
  pub fn on_cheat_set(&mut self, index: libc::c_uint, enabled: bool, code: *const libc::c_char) {
    self.core_mut(|core| core.cheat_set(index, enabled, code))
  }

  /// Invoked by a `libretro` frontend, with the `retro_load_game` API call.
  pub fn on_load_game(&mut self, game: &retro_game_info) -> bool {
    self.core_mut(|core| core.load_game(game.into()))
  }

  /// Invoked by a `libretro` frontend, with the `retro_load_game_special` API call.
  pub fn on_load_game_special(&mut self, game_type: libc::c_uint, info: &retro_game_info, num_info: libc::size_t) -> bool {
    self.core_mut(|core| core.load_game_special(game_type, info.into(), num_info))
  }

  /// Invoked by a `libretro` frontend, with the `retro_unload_game` API call.
  pub fn on_unload_game(&mut self) {
    self.core_mut(|core| core.unload_game())
  }

  /// Invoked by a `libretro` frontend, with the `retro_get_region` API call.
  pub fn on_get_region(&self) -> libc::c_uint {
    self.core_ref(|core| core.get_region().into())
  }

  /// Invoked by a `libretro` frontend, with the `retro_get_memory_data` API call.
  pub fn on_get_memory_data(&mut self, id: libc::c_uint) -> *mut () {
    self.core_mut(|core| core.get_memory_data(id))
  }

  /// Invoked by a `libretro` frontend, with the `retro_get_memory_size` API call.
  pub fn on_get_memory_size(&mut self, id: libc::c_uint) -> libc::size_t {
    self.core_mut(|core| core.get_memory_size(id))
  }

  #[inline]
  #[doc(hidden)]
  fn core_mut<F, Output>(&mut self, f: F) -> Output
  where
    F: FnOnce(&mut T) -> Output,
  {
    f(self.core.as_mut().unwrap())
  }

  #[inline]
  #[doc(hidden)]
  fn core_ref<F, Output>(&self, f: F) -> Output
  where
    F: FnOnce(&T) -> Output,
  {
    f(self.core.as_ref().unwrap())
  }
}
