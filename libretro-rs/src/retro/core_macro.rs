use crate::ffi::*;
use crate::prelude::*;
use core::ffi::{c_char, c_uint, CStr};

/// This is the glue layer between a [Core] and the `libretro` API.
#[doc(hidden)]
pub struct Instance<T> {
  pub system: Option<T>,
  pub audio_sample: retro_audio_sample_t,
  pub audio_sample_batch: retro_audio_sample_batch_t,
  pub environment: retro_environment_t,
  pub input_poll: retro_input_poll_t,
  pub input_state: retro_input_state_t,
  pub video_refresh: retro_video_refresh_t,
}

impl<T: Core> Instance<T> {
  /// Invoked by a `libretro` frontend, with the `retro_get_system_info` API call.
  pub fn on_get_system_info(&mut self, info: &mut retro_system_info) {
    *info = T::get_system_info().into();
  }

  /// Invoked by a `libretro` frontend, with the `retro_get_system_av_info` API call.
  pub fn on_get_system_av_info(&self, info: &mut retro_system_av_info) {
    let system = self
      .system
      .as_ref()
      .expect("`retro_get_system_av_info` called without a successful `retro_load_game` call. The frontend is not compliant");
    *info = system.get_system_av_info(&mut self.environment()).into();
  }

  /// Invoked by a `libretro` frontend, with the `retro_init` API call.
  pub fn on_init(&self) {
    T::init(&mut self.environment());
  }

  /// Invoked by a `libretro` frontend, with the `retro_deinit` API call.
  pub fn on_deinit(&mut self) {
    self.system = None;
    self.audio_sample = None;
    self.audio_sample_batch = None;
    self.environment = None;
    self.input_poll = None;
    self.input_state = None;
    self.video_refresh = None;
  }

  /// Invoked by a `libretro` frontend, with the `retro_set_environment` API call.
  pub fn on_set_environment(&mut self, mut env: EnvironmentCallback) {
    T::set_environment(&mut env);

    self.environment = Some(env);
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
  pub fn on_set_controller_port_device(&mut self, port: c_uint, device: c_uint) {
    if let Ok(device) = device.try_into() {
      if let Ok(port_num) = u8::try_from(port) {
        let mut env = self.environment();
        let port = DevicePort::new(port_num);
        self.core_mut(|core| core.set_controller_port_device(&mut env, port, device))
      }
    }
  }

  /// Invoked by a `libretro` frontend, with the `retro_reset` API call.
  pub fn on_reset(&mut self) {
    let mut env = self.environment();
    self.core_mut(|core| core.reset(&mut env))
  }

  /// Invoked by a `libretro` frontend, with the `retro_run` API call.
  pub fn on_run(&mut self) {
    // `input_poll` is required to be called once per `retro_run`.
    self.input_poll();

    let mut env = self.environment();

    let runtime = Runtime::new(
      self.audio_sample,
      self.audio_sample_batch,
      self.input_state,
      self.video_refresh,
    );

    self.core_mut(|core| core.run(&mut env, &runtime));
  }

  fn input_poll(&mut self) {
    let cb = self
      .input_poll
      .expect("`on_run` called without registering an `input_poll` callback");

    unsafe { cb() }
  }

  /// Invoked by a `libretro` frontend, with the `retro_serialize_size` API call.
  pub fn on_serialize_size(&self) -> usize {
    let mut env = self.environment();
    self.core_ref(|core| core.serialize_size(&mut env))
  }

  /// Invoked by a `libretro` frontend, with the `retro_serialize` API call.
  pub fn on_serialize(&self, data: *mut (), size: usize) -> bool {
    unsafe {
      let data = core::slice::from_raw_parts_mut(data as *mut u8, size);
      let mut env = self.environment();
      self.core_ref(|core| core.serialize(&mut env, data).is_ok())
    }
  }

  /// Invoked by a `libretro` frontend, with the `retro_unserialize` API call.
  pub fn on_unserialize(&mut self, data: *const (), size: usize) -> bool {
    unsafe {
      let data = core::slice::from_raw_parts(data as *const u8, size);
      let mut env = self.environment();
      self.core_mut(|core| core.unserialize(&mut env, data).is_ok())
    }
  }

  /// Invoked by a `libretro` frontend, with the `retro_cheat_reset` API call.
  pub fn on_cheat_reset(&mut self) {
    let mut env = self.environment();
    self.core_mut(|core| core.cheat_reset(&mut env))
  }

  /// Invoked by a `libretro` frontend, with the `retro_cheat_set` API call.
  ///
  /// # Safety
  /// `code` must be a valid argument to [`CStr::from_ptr`].
  pub unsafe fn on_cheat_set(&mut self, index: c_uint, enabled: bool, code: *const c_char) {
    unsafe {
      let code = CStr::from_ptr(code).to_str().expect("`code` contains invalid data");
      let mut env = self.environment();
      self.core_mut(|core| core.cheat_set(&mut env, index, enabled, code))
    }
  }

  /// Invoked by a `libretro` frontend, with the `retro_load_game` API call.
  ///
  /// # Safety
  /// `game` must remain valid until [`Instance::on_unload_game`] is called.
  pub unsafe fn on_load_game(&mut self, game: *const retro_game_info) -> bool {
    let mut env = self.environment();
    let game = game.as_ref().map_or_else(Game::default, Game::from);
    self.system = T::load_game(&mut env, game).ok();
    self.system.is_some()
  }

  /// Invoked by a `libretro` frontend, with the `retro_load_game_special` API call.
  pub fn on_load_game_special(&mut self, game_type: GameType, info: &retro_game_info, _num_info: usize) -> bool {
    let mut env = self.environment();
    self.system = T::load_game_special(&mut env, game_type, info.into()).ok();
    self.system.is_some()
  }

  /// Invoked by a `libretro` frontend, with the `retro_unload_game` API call.
  pub fn on_unload_game(&mut self) {
    let mut env = self.environment();
    self.core_mut(|core| core.unload_game(&mut env))
  }

  /// Invoked by a `libretro` frontend, with the `retro_get_region` API call.
  pub fn on_get_region(&self) -> c_uint {
    let system = self.system.as_ref().expect("`on_get_region` called without a game loaded.");
    c_uint::from(system.get_region(&mut self.environment()))
  }

  /// Invoked by a `libretro` frontend, with the `retro_get_memory_data` API call.
  pub fn on_get_memory_data(&mut self, id: MemoryType) -> *mut () {
    let mut env = self.environment();
    self.core_mut(|core| {
      core
        .get_memory_data(&mut env, id)
        .map_or_else(std::ptr::null_mut, |data| data.as_mut_ptr() as *mut ())
    })
  }

  /// Invoked by a `libretro` frontend, with the `retro_get_memory_size` API call.
  pub fn on_get_memory_size(&mut self, id: MemoryType) -> usize {
    let mut env = self.environment();
    self.core_mut(|core| core.get_memory_data(&mut env, id).map_or(0, |data| data.len()))
  }

  #[inline]
  #[doc(hidden)]
  fn environment(&self) -> EnvironmentCallback {
    self.environment.expect("unable to retrieve the environment callback")
  }

  #[inline]
  #[doc(hidden)]
  fn core_mut<F, Output>(&mut self, f: F) -> Output
  where
    F: FnOnce(&mut T) -> Output,
  {
    let sys = self
      .system
      .as_mut()
      .expect("`core_mut` called when no system has been created");

    f(sys)
  }

  #[inline]
  #[doc(hidden)]
  fn core_ref<F, Output>(&self, f: F) -> Output
  where
    F: FnOnce(&T) -> Output,
  {
    let sys = self
      .system
      .as_ref()
      .expect("`core_ref` called when no system has been created");

    f(sys)
  }
}

#[macro_export]
macro_rules! libretro_core {
  ($core:ty) => {
    #[doc(hidden)]
    mod __libretro_rs_gen {
      use super::*;
      use core::ffi::c_char;
      use core::ffi::*;
      use libretro_rs::ffi::*;
      use libretro_rs::prelude::*;

      static mut RETRO_INSTANCE: Instance<$core> = Instance {
        environment: None,
        audio_sample: None,
        audio_sample_batch: None,
        input_poll: None,
        input_state: None,
        video_refresh: None,
        system: None,
      };

      #[no_mangle]
      extern "C" fn retro_api_version() -> c_uint {
        RETRO_API_VERSION
      }

      #[no_mangle]
      extern "C" fn retro_get_system_info(info: &mut retro_system_info) {
        instance_mut(|instance| instance.on_get_system_info(info))
      }

      #[no_mangle]
      extern "C" fn retro_get_system_av_info(info: &mut retro_system_av_info) {
        instance_ref(|instance| instance.on_get_system_av_info(info))
      }

      #[no_mangle]
      extern "C" fn retro_init() {
        instance_ref(|instance| instance.on_init())
      }

      #[no_mangle]
      extern "C" fn retro_deinit() {
        instance_mut(|instance| instance.on_deinit())
      }

      #[no_mangle]
      extern "C" fn retro_set_environment(cb: EnvironmentCallback) {
        instance_mut(|instance| instance.on_set_environment(cb))
      }

      #[no_mangle]
      extern "C" fn retro_set_audio_sample(cb: retro_audio_sample_t) {
        instance_mut(|instance| instance.on_set_audio_sample(cb))
      }

      #[no_mangle]
      extern "C" fn retro_set_audio_sample_batch(cb: retro_audio_sample_batch_t) {
        instance_mut(|instance| instance.on_set_audio_sample_batch(cb))
      }

      #[no_mangle]
      extern "C" fn retro_set_input_poll(cb: retro_input_poll_t) {
        instance_mut(|instance| instance.on_set_input_poll(cb))
      }

      #[no_mangle]
      extern "C" fn retro_set_input_state(cb: retro_input_state_t) {
        instance_mut(|instance| instance.on_set_input_state(cb))
      }

      #[no_mangle]
      extern "C" fn retro_set_video_refresh(cb: retro_video_refresh_t) {
        instance_mut(|instance| instance.on_set_video_refresh(cb))
      }

      #[no_mangle]
      extern "C" fn retro_set_controller_port_device(port: c_uint, device: c_uint) {
        instance_mut(|instance| instance.on_set_controller_port_device(port, device))
      }

      #[no_mangle]
      extern "C" fn retro_reset() {
        instance_mut(|instance| instance.on_reset())
      }

      #[no_mangle]
      extern "C" fn retro_run() {
        instance_mut(|instance| instance.on_run())
      }

      #[no_mangle]
      extern "C" fn retro_serialize_size() -> usize {
        instance_ref(|instance| instance.on_serialize_size())
      }

      #[no_mangle]
      extern "C" fn retro_serialize(data: *mut (), size: usize) -> bool {
        instance_ref(|instance| instance.on_serialize(data, size))
      }

      #[no_mangle]
      extern "C" fn retro_unserialize(data: *const (), size: usize) -> bool {
        instance_mut(|instance| instance.on_unserialize(data, size))
      }

      #[no_mangle]
      extern "C" fn retro_cheat_reset() {
        instance_mut(|instance| instance.on_cheat_reset())
      }

      #[no_mangle]
      unsafe extern "C" fn retro_cheat_set(index: c_uint, enabled: bool, code: *const c_char) {
        instance_mut(|instance| instance.on_cheat_set(index, enabled, code))
      }

      #[no_mangle]
      unsafe extern "C" fn retro_load_game(game: *const retro_game_info) -> bool {
        instance_mut(|instance| instance.on_load_game(game))
      }

      #[no_mangle]
      extern "C" fn retro_load_game_special(game_type: GameType, info: &retro_game_info, num_info: usize) -> bool {
        instance_mut(|instance| instance.on_load_game_special(game_type, info, num_info))
      }

      #[no_mangle]
      extern "C" fn retro_unload_game() {
        instance_mut(|instance| instance.on_unload_game())
      }

      #[no_mangle]
      extern "C" fn retro_get_region() -> c_uint {
        instance_ref(|instance| instance.on_get_region())
      }

      #[no_mangle]
      extern "C" fn retro_get_memory_data(id: MemoryType) -> *mut () {
        instance_mut(|instance| instance.on_get_memory_data(id))
      }

      #[no_mangle]
      extern "C" fn retro_get_memory_size(id: MemoryType) -> usize {
        instance_mut(|instance| instance.on_get_memory_size(id))
      }

      #[inline]
      fn instance_ref<T>(f: impl FnOnce(&Instance<$core>) -> T) -> T {
        unsafe { f(&RETRO_INSTANCE) }
      }

      #[inline]
      fn instance_mut<T>(f: impl FnOnce(&mut Instance<$core>) -> T) -> T {
        unsafe { f(&mut RETRO_INSTANCE) }
      }
    }
  };
}
