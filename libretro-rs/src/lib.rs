#![allow(unused_variables)]

use std::ffi::CStr;

pub use libc;

pub mod sys;

use libc::c_void;
use sys::*;

pub trait RetroCore {
  fn new(env: &RetroEnvironment) -> Self;

  fn get_system_info(info: &mut crate::sys::retro_system_info);

  fn get_system_av_info(&self, info: &mut crate::sys::retro_system_av_info);

  fn set_controller_port_device(&mut self, port: u32, device: u32);

  fn reset(&mut self);

  fn run(&mut self);

  fn serialize_size(&self) -> usize { 0 }

  fn serialize(&self, data: *mut (), size: usize) -> bool { false }

  fn unserialize(&mut self, data: *const (), size: usize) -> bool { false }

  fn cheat_reset(&mut self) {}

  fn cheat_set(&mut self, index: u32, enabled: bool, code: *const libc::c_char) {}

  fn load_game(&mut self, game: &sys::retro_game_info);

  fn load_game_special(&mut self, game_type: u32, info: &sys::retro_game_info, num_info: usize) {
  }

  fn unload_game(&mut self) {
  }

  fn get_region(&mut self) -> u32 {
    RETRO_REGION_NTSC
  }

  fn get_memory_data(&mut self, id: u32) -> *mut () {
    std::ptr::null_mut()
  }

  fn get_memory_size(&mut self, id: u32) -> usize {
    0
  }
}

pub struct RetroEnvironment(retro_environment_t);

impl RetroEnvironment {
  pub fn new(cb: retro_environment_t) -> RetroEnvironment {
    RetroEnvironment(cb)
  }

  pub fn get_system_directory(&self) -> Option<String> {
    self.get_string(RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY)
  }

  fn get_string(&self, key: u32) -> Option<String> {
    unsafe {
      let s = self.get(key)?;
      let s = CStr::from_ptr(s).to_str().ok()?;
      Some(s.into())
    }
  }

  unsafe fn get<T>(&self, key: u32) -> Option<*const T> {
    let mut val = std::ptr::null();
    if self.get_raw(key, &mut val) {
      Some(val)
    } else {
      None
    }
  }

  unsafe fn get_raw<T>(&self, key: u32, output: *mut *const T) -> bool {
    self.0.unwrap()(key, output as *mut c_void)
  }
}

#[macro_export]
macro_rules! libretro_core {
  ($core:path) => {
    static mut RETRO_CONTEXT: RetroContext<$core> = RetroContext {
      environment: None,
      audio_sample: None,
      audio_sample_batch: None,
      input_poll: None,
      input_state: None,
      video_refresh: None,
      core: None,
    };

    struct RetroContext<T: RetroCore> {
      environment: libretro_rs::sys::retro_environment_t,
      audio_sample: libretro_rs::sys::retro_audio_sample_t,
      audio_sample_batch: libretro_rs::sys::retro_audio_sample_batch_t,
      input_poll: libretro_rs::sys::retro_input_poll_t,
      input_state: libretro_rs::sys::retro_input_state_t,
      video_refresh: libretro_rs::sys::retro_video_refresh_t,
      core: Option<T>,
    }

    #[no_mangle]
    unsafe extern "C" fn retro_api_version() -> libretro_rs::libc::c_uint {
      libretro_rs::sys::RETRO_API_VERSION
    }

    #[no_mangle]
    unsafe extern "C" fn retro_get_system_info(info: &mut libretro_rs::sys::retro_system_info) {
      <$core>::get_system_info(info)
    }

    #[no_mangle]
    unsafe extern "C" fn retro_get_system_av_info(info: &mut libretro_rs::sys::retro_system_av_info) {
      RETRO_CONTEXT.core.as_ref().unwrap().get_system_av_info(info)
    }

    #[no_mangle]
    unsafe extern "C" fn retro_init() {
      let env = libretro_rs::RetroEnvironment::new(RETRO_CONTEXT.environment);
      RETRO_CONTEXT.core = Some(<$core>::new(&env))
    }

    #[no_mangle]
    unsafe extern "C" fn retro_deinit() {
      RETRO_CONTEXT.environment = None;
      RETRO_CONTEXT.audio_sample = None;
      RETRO_CONTEXT.audio_sample_batch = None;
      RETRO_CONTEXT.input_poll = None;
      RETRO_CONTEXT.input_state = None;
      RETRO_CONTEXT.video_refresh = None;
      RETRO_CONTEXT.core = None;
    }

    #[no_mangle]
    unsafe extern "C" fn retro_set_environment(cb: libretro_rs::sys::retro_environment_t) {
      RETRO_CONTEXT.environment = cb
    }

    #[no_mangle]
    unsafe extern "C" fn retro_set_audio_sample(cb: libretro_rs::sys::retro_audio_sample_t) {
      RETRO_CONTEXT.audio_sample = cb
    }

    #[no_mangle]
    unsafe extern "C" fn retro_set_audio_sample_batch(cb: libretro_rs::sys::retro_audio_sample_batch_t) {
      RETRO_CONTEXT.audio_sample_batch = cb
    }

    #[no_mangle]
    unsafe extern "C" fn retro_set_input_poll(cb: libretro_rs::sys::retro_input_poll_t) {
      RETRO_CONTEXT.input_poll = cb
    }

    #[no_mangle]
    unsafe extern "C" fn retro_set_input_state(cb: libretro_rs::sys::retro_input_state_t) {
      RETRO_CONTEXT.input_state = cb
    }

    #[no_mangle]
    unsafe extern "C" fn retro_set_video_refresh(cb: libretro_rs::sys::retro_video_refresh_t) {
      RETRO_CONTEXT.video_refresh = cb
    }

    #[no_mangle]
    unsafe extern "C" fn retro_set_controller_port_device(port: libretro_rs::libc::c_uint, device: libretro_rs::libc::c_uint) {
      RETRO_CONTEXT.core.as_mut().unwrap().set_controller_port_device(port, device)
    }

    #[no_mangle]
    unsafe extern "C" fn retro_reset() {
      RETRO_CONTEXT.core.as_mut().unwrap().reset()
    }

    #[no_mangle]
    unsafe extern "C" fn retro_run() {
      RETRO_CONTEXT.core.as_mut().unwrap().run()
    }

    #[no_mangle]
    unsafe extern "C" fn retro_serialize_size() -> libretro_rs::libc::size_t {
      RETRO_CONTEXT.core.as_ref().unwrap().serialize_size()
    }

    #[no_mangle]
    unsafe extern "C" fn retro_serialize(data: *mut (), size: libretro_rs::libc::size_t) -> bool {
      RETRO_CONTEXT.core.as_ref().unwrap().serialize(data, size)
    }

    #[no_mangle]
    unsafe extern "C" fn retro_unserialize(data: *const (), size: libretro_rs::libc::size_t) -> bool {
      RETRO_CONTEXT.core.as_mut().unwrap().unserialize(data, size)
    }

    #[no_mangle]
    unsafe extern "C" fn retro_cheat_reset() {
      RETRO_CONTEXT.core.as_mut().unwrap().cheat_reset()
    }

    #[no_mangle]
    unsafe extern "C" fn retro_cheat_set(index: libretro_rs::libc::c_uint, enabled: bool, code: *const libretro_rs::libc::c_char) {
      RETRO_CONTEXT.core.as_mut().unwrap().cheat_set(index, enabled, code)
    }

    #[no_mangle]
    unsafe extern "C" fn retro_load_game(game: &libretro_rs::sys::retro_game_info) {
      RETRO_CONTEXT.core.as_mut().unwrap().load_game(game)
    }

    #[no_mangle]
    unsafe extern "C" fn retro_load_game_special(game_type: libretro_rs::libc::c_uint, info: &libretro_rs::sys::retro_game_info, num_info: libretro_rs::libc::size_t) {
      RETRO_CONTEXT.core.as_mut().unwrap().load_game_special(game_type, info, num_info)
    }

    #[no_mangle]
    unsafe extern "C" fn retro_unload_game() {
      RETRO_CONTEXT.core.as_mut().unwrap().unload_game()
    }

    #[no_mangle]
    unsafe extern "C" fn retro_get_region() -> libretro_rs::libc::c_uint {
      RETRO_CONTEXT.core.as_mut().unwrap().get_region()
    }

    #[no_mangle]
    unsafe extern "C" fn retro_get_memory_data(id: libretro_rs::libc::c_uint) -> *mut () {
      RETRO_CONTEXT.core.as_mut().unwrap().get_memory_data(id)
    }

    #[no_mangle]
    unsafe extern "C" fn retro_get_memory_size(id: libretro_rs::libc::c_uint) -> libretro_rs::libc::size_t {
      RETRO_CONTEXT.core.as_mut().unwrap().get_memory_size(id)
    }
  }
}
