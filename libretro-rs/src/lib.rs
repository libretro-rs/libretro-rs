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

  fn get_region(&self) -> u32 {
    RETRO_REGION_NTSC
  }

  fn get_memory_data(&mut self, id: u32) -> *mut () {
    std::ptr::null_mut()
  }

  fn get_memory_size(&self, id: u32) -> usize {
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
    extern "C" fn retro_api_version() -> libretro_rs::libc::c_uint {
      libretro_rs::sys::RETRO_API_VERSION
    }

    #[no_mangle]
    extern "C" fn retro_get_system_info(info: &mut libretro_rs::sys::retro_system_info) {
      <$core>::get_system_info(info)
    }

    #[no_mangle]
    extern "C" fn retro_get_system_av_info(info: &mut libretro_rs::sys::retro_system_av_info) {
      core_ref(|core| core.get_system_av_info(info))
    }

    #[no_mangle]
    extern "C" fn retro_init() {
      context_mut(|ctx| {
        let env = libretro_rs::RetroEnvironment::new(ctx.environment);
        ctx.core = Some(<$core>::new(&env))
      })
    }

    #[no_mangle]
    extern "C" fn retro_deinit() {
      context_mut(|ctx| {
        ctx.environment = None;
        ctx.audio_sample = None;
        ctx.audio_sample_batch = None;
        ctx.input_poll = None;
        ctx.input_state = None;
        ctx.video_refresh = None;
        ctx.core = None;
      })
    }

    #[no_mangle]
    extern "C" fn retro_set_environment(cb: libretro_rs::sys::retro_environment_t) {
      context_mut(|ctx| ctx.environment = cb)
    }

    #[no_mangle]
    extern "C" fn retro_set_audio_sample(cb: libretro_rs::sys::retro_audio_sample_t) {
      context_mut(|ctx| ctx.audio_sample = cb)
    }

    #[no_mangle]
    extern "C" fn retro_set_audio_sample_batch(cb: libretro_rs::sys::retro_audio_sample_batch_t) {
      context_mut(|ctx| ctx.audio_sample_batch = cb)
    }

    #[no_mangle]
    extern "C" fn retro_set_input_poll(cb: libretro_rs::sys::retro_input_poll_t) {
      context_mut(|ctx| ctx.input_poll = cb)
    }

    #[no_mangle]
    extern "C" fn retro_set_input_state(cb: libretro_rs::sys::retro_input_state_t) {
      context_mut(|ctx| ctx.input_state = cb)
    }

    #[no_mangle]
    extern "C" fn retro_set_video_refresh(cb: libretro_rs::sys::retro_video_refresh_t) {
      context_mut(|ctx| ctx.video_refresh = cb)
    }

    #[no_mangle]
    extern "C" fn retro_set_controller_port_device(port: libretro_rs::libc::c_uint, device: libretro_rs::libc::c_uint) {
      core_mut(|core| core.set_controller_port_device(port, device))
    }

    #[no_mangle]
    extern "C" fn retro_reset() {
      core_mut(|core| core.reset())
    }

    #[no_mangle]
    extern "C" fn retro_run() {
      core_mut(|core| core.run())
    }

    #[no_mangle]
    extern "C" fn retro_serialize_size() -> libretro_rs::libc::size_t {
      core_ref(|core| core.serialize_size())
    }

    #[no_mangle]
    extern "C" fn retro_serialize(data: *mut (), size: libretro_rs::libc::size_t) -> bool {
      core_ref(|core| core.serialize(data, size))
    }

    #[no_mangle]
    extern "C" fn retro_unserialize(data: *const (), size: libretro_rs::libc::size_t) -> bool {
      core_mut(|core| core.unserialize(data, size))
    }

    #[no_mangle]
    extern "C" fn retro_cheat_reset() {
      core_mut(|core| core.cheat_reset())
    }

    #[no_mangle]
    extern "C" fn retro_cheat_set(index: libretro_rs::libc::c_uint, enabled: bool, code: *const libretro_rs::libc::c_char) {
      core_mut(|core| core.cheat_set(index, enabled, code))
    }

    #[no_mangle]
    extern "C" fn retro_load_game(game: &libretro_rs::sys::retro_game_info) {
      core_mut(|core| core.load_game(game))
    }

    #[no_mangle]
    extern "C" fn retro_load_game_special(game_type: libretro_rs::libc::c_uint, info: &libretro_rs::sys::retro_game_info, num_info: libretro_rs::libc::size_t) {
      core_mut(|core| core.load_game_special(game_type, info, num_info))
    }

    #[no_mangle]
    extern "C" fn retro_unload_game() {
      core_mut(|core| core.unload_game())
    }

    #[no_mangle]
    extern "C" fn retro_get_region() -> libretro_rs::libc::c_uint {
      core_ref(|core| core.get_region())
    }

    #[no_mangle]
    extern "C" fn retro_get_memory_data(id: libretro_rs::libc::c_uint) -> *mut () {
      core_mut(|core| core.get_memory_data(id))
    }

    #[no_mangle]
    extern "C" fn retro_get_memory_size(id: libretro_rs::libc::c_uint) -> libretro_rs::libc::size_t {
      core_ref(|core| core.get_memory_size(id))
    }

    #[inline]
    fn core_ref<T>(f: impl FnOnce(&$core) -> T) -> T {
      context_ref(|ctx| f(ctx.core.as_ref().unwrap()))
    }

    #[inline]
    fn core_mut<T>(f: impl FnOnce(&mut $core) -> T) -> T {
      context_mut(|ctx| (f)(ctx.core.as_mut().unwrap()))
    }

    #[inline]
    fn context_ref<T>(f: impl FnOnce(&RetroContext<$core>) -> T) -> T {
      unsafe {
        (f)(&RETRO_CONTEXT)
      }
    }

    #[inline]
    fn context_mut<T>(f: impl FnOnce(&mut RetroContext<$core>) -> T) -> T {
      unsafe {
        (f)(&mut RETRO_CONTEXT)
      }
    }
  }
}
