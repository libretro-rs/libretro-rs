#![allow(unused_variables)]

use std::ffi::CStr;

pub use libc;

pub mod core_macro;
pub mod sys;

use libc::c_void;
use sys::*;

pub trait RetroCore {
  fn new(env: &RetroEnvironment) -> Self;

  fn get_system_info(info: &mut retro_system_info);

  fn get_system_av_info(&self, info: &mut retro_system_av_info);

  fn set_controller_port_device(&mut self, port: u32, device: RetroDevice);

  fn reset(&mut self);

  fn run(&mut self);

  fn serialize_size(&self) -> usize {
    0
  }

  fn serialize(&self, data: *mut (), size: usize) -> bool {
    false
  }

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

pub struct RetroEnvironment(retro_environment_t);

impl RetroEnvironment {
  pub fn new(cb: retro_environment_t) -> RetroEnvironment {
    RetroEnvironment(cb)
  }

  /* Commands */

  pub fn shutdown(&self) -> bool {
    unsafe {
      (self.0.unwrap())(RETRO_ENVIRONMENT_SHUTDOWN, std::ptr::null_mut())
    }
  }

  /* Queries */

  pub fn get_libretro_path(&self) -> Option<&str> {
    self.get_str(RETRO_ENVIRONMENT_GET_LIBRETRO_PATH)
  }

  pub fn get_core_assets_directory(&self) -> Option<&str> {
    self.get_str(RETRO_ENVIRONMENT_GET_CORE_ASSETS_DIRECTORY)
  }

  pub fn get_save_directory(&self) -> Option<&str> {
    self.get_str(RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY)
  }

  pub fn get_system_directory(&self) -> Option<&str> {
    self.get_str(RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY)
  }

  pub fn get_username(&self) -> Option<&str> {
    self.get_str(RETRO_ENVIRONMENT_GET_USERNAME)
  }

  fn get_str<'a>(&'a self, key: u32) -> Option<&'a str> {
    unsafe {
      let s = self.get(key)?;
      CStr::from_ptr(s).to_str().ok()
    }
  }

  unsafe fn get<T>(&self, key: u32) -> Option<*const T> {
    let mut val: *const T = std::ptr::null();
    if self.get_raw(key, &mut val) && !val.is_null() {
      Some(val)
    } else {
      None
    }
  }

  unsafe fn get_raw<T>(&self, key: u32, output: *mut *const T) -> bool {
    self.0.unwrap()(key, output as *mut c_void)
  }
}

pub enum RetroGame<'a> {
  None,
  Data(&'a [u8]),
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

pub enum RetroRegion {
  NTSC = 0,
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
  pub fn on_get_system_info(&self, info: &mut retro_system_info) {
    T::get_system_info(info)
  }

  pub fn on_get_system_av_info(&self, info: &mut retro_system_av_info) {
    self.core_ref(|core| core.get_system_av_info(info))
  }

  pub fn on_init(&mut self) {
    let env = RetroEnvironment::new(self.environment);
    self.core = Some(T::new(&env))
  }

  pub fn on_deinit(&mut self) {
    self.core = None;
    self.audio_sample = None;
    self.audio_sample_batch = None;
    self.environment = None;
    self.input_poll = None;
    self.input_state = None;
    self.video_refresh = None;
  }

  pub fn on_set_environment(&mut self, cb: retro_environment_t) {
    self.environment = cb;
  }

  pub fn on_set_audio_sample(&mut self, cb: retro_audio_sample_t) {
    self.audio_sample = cb;
  }

  pub fn on_set_audio_sample_batch(&mut self, cb: retro_audio_sample_batch_t) {
    self.audio_sample_batch = cb;
  }

  pub fn on_set_input_poll(&mut self, cb: retro_input_poll_t) {
    self.input_poll = cb;
  }

  pub fn on_set_input_state(&mut self, cb: retro_input_state_t) {
    self.input_state = cb;
  }

  pub fn on_set_video_refresh(&mut self, cb: retro_video_refresh_t) {
    self.video_refresh = cb;
  }

  pub fn on_set_controller_port_device(&mut self, port: libc::c_uint, device: libc::c_uint) {
    self.core_mut(|core| core.set_controller_port_device(port, device.into()))
  }

  pub fn on_reset(&mut self) {
    self.core_mut(|core| core.reset())
  }

  pub fn on_run(&mut self) {
    self.core_mut(|core| core.run())
  }

  pub fn on_serialize_size(&self) -> libc::size_t {
    self.core_ref(|core| core.serialize_size())
  }

  pub fn on_serialize(&self, data: *mut (), size: libc::size_t) -> bool {
    self.core_ref(|core| core.serialize(data, size))
  }

  pub fn on_unserialize(&mut self, data: *const (), size: libc::size_t) -> bool {
    self.core_mut(|core| core.unserialize(data, size))
  }

  pub fn on_cheat_reset(&mut self) {
    self.core_mut(|core| core.cheat_reset())
  }

  pub fn on_cheat_set(&mut self, index: libc::c_uint, enabled: bool, code: *const libc::c_char) {
    self.core_mut(|core| core.cheat_set(index, enabled, code))
  }

  pub fn on_load_game(&mut self, game: &retro_game_info) -> bool {
    self.core_mut(|core| core.load_game(game.into()))
  }

  pub fn on_load_game_special(&mut self, game_type: libc::c_uint, info: &retro_game_info, num_info: libc::size_t) -> bool {
    self.core_mut(|core| core.load_game_special(game_type, info.into(), num_info))
  }

  pub fn on_unload_game(&mut self) {
    self.core_mut(|core| core.unload_game())
  }

  pub fn on_get_region(&self) -> libc::c_uint {
    self.core_ref(|core| core.get_region().into())
  }

  pub fn on_get_memory_data(&mut self, id: libc::c_uint) -> *mut () {
    self.core_mut(|core| core.get_memory_data(id))
  }

  pub fn on_get_memory_size(&mut self, id: libc::c_uint) -> libc::size_t {
    self.core_mut(|core| core.get_memory_size(id))
  }

  #[inline]
  fn core_mut<F, Output>(&mut self, f: F) -> Output
  where
    F: FnOnce(&mut T) -> Output,
  {
    f(self.core.as_mut().unwrap())
  }

  #[inline]
  fn core_ref<F, Output>(&self, f: F) -> Output
  where
    F: FnOnce(&T) -> Output,
  {
    f(self.core.as_ref().unwrap())
  }
}
