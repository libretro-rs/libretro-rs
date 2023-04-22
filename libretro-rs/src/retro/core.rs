use crate::ffi::*;
use crate::retro::*;
use ::core::ffi::*;
use ::core::ops::*;
use c_utf8::CUtf8;

#[allow(unused_variables)]
pub trait Core: Sized {
  /// Called during `retro_set_environment`.
  fn set_environment(env: &mut impl env::SetEnvironment) {}

  /// Called during `retro_init`. This function is provided for the sake of completeness; it's generally redundant
  /// with [`Core::load_game`].
  fn init(env: &mut impl env::Init) {}

  /// Called to get information about the core. This information can then be displayed in a frontend, or used to
  /// construct core-specific paths.
  fn get_system_info() -> SystemInfo;

  fn get_system_av_info(&self, env: &mut impl env::GetAvInfo) -> SystemAVInfo;

  fn get_region(&self, env: &mut impl env::GetRegion) -> Region {
    Region::NTSC
  }

  /// Called to associate a particular device with a particular port. A core is allowed to ignore this request.
  fn set_controller_port_device(&mut self, env: &mut impl env::SetPortDevice, port: DevicePort, device: Device) {}

  /// Called when a player resets their game.
  fn reset(&mut self, env: &mut impl env::Reset);

  /// Called continuously once the core is initialized and a game is loaded. The core is expected to advance emulation
  /// by a single frame before returning.
  fn run(&mut self, env: &mut impl env::Run, runtime: &Runtime);

  /// Called to determine the size of the save state buffer. This is only ever called once per run, and the core must
  /// not exceed the size returned here for subsequent saves.
  fn serialize_size(&self, env: &mut impl env::SerializeSize) -> usize {
    0
  }

  /// Allows a core to save its internal state into the specified buffer. The buffer is guaranteed to be at least `size`
  /// bytes, where `size` is the value returned from `serialize_size`.
  fn serialize(&self, env: &mut impl env::Serialize, data: &mut [u8]) -> Result<(), SerializeError> {
    Err(SerializeError::new())
  }

  /// Allows a core to load its internal state from the specified buffer. The buffer is guaranteed to be at least `size`
  /// bytes, where `size` is the value returned from `serialize_size`.
  fn unserialize(&mut self, env: &mut impl env::Unserialize, data: &[u8]) -> Result<(), UnserializeError> {
    Err(UnserializeError::new())
  }

  fn cheat_reset(&mut self, env: &mut impl env::CheatReset) {}

  fn cheat_set(&mut self, env: &mut impl env::CheatSet, index: u32, enabled: bool, code: &str) {}

  /// Called when a new instance of the core is needed. The `env` parameter can be used to set-up and/or query values
  /// required for the core to function properly.
  fn load_game(env: &mut impl env::LoadGame, game: Game) -> Result<Self, LoadGameError>;

  fn load_game_special(env: &mut impl env::LoadGameSpecial, game_type: GameType, info: Game) -> Result<Self, LoadGameError> {
    Err(LoadGameError::new())
  }

  fn unload_game(&mut self, env: &mut impl env::UnloadGame) {}

  fn get_memory_data(&mut self, env: &mut impl env::GetMemoryData, id: MemoryType) -> Option<&mut [u8]> {
    None
  }
}

/// Rust interface for [`retro_system_info`].
#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct SystemInfo(retro_system_info);

impl SystemInfo {
  /// Minimal constructor. Leaves [`SystemInfo::need_fullpath`] and
  /// [`SystemInfo::block_extract`] set to [false].
  pub fn new(library_name: &'static CUtf8, library_version: &'static CUtf8, valid_extensions: Extensions) -> Self {
    Self(retro_system_info {
      library_name: library_name.as_ptr(),
      library_version: library_version.as_ptr(),
      valid_extensions: valid_extensions.as_ptr(),
      need_fullpath: false,
      block_extract: false,
    })
  }

  pub fn with_block_extract(mut self) -> Self {
    self.0.block_extract = true;
    self
  }

  pub fn with_need_full_path(mut self) -> Self {
    self.0.need_fullpath = true;
    self
  }

  pub fn library_name(&self) -> &'static CUtf8 {
    unsafe { Self::ptr_to_str(self.0.library_name) }
  }

  pub fn library_version(&self) -> &'static CUtf8 {
    unsafe { Self::ptr_to_str(self.0.library_version) }
  }

  pub fn valid_extensions(&self) -> Extensions {
    if self.0.valid_extensions.is_null() {
      Extensions(None)
    } else {
      Extensions(Some(unsafe { Self::ptr_to_str(self.0.valid_extensions) }))
    }
  }

  pub fn need_fullpath(&self) -> bool {
    self.0.need_fullpath
  }

  pub fn block_extract(&self) -> bool {
    self.0.block_extract
  }

  pub fn into_inner(self) -> retro_system_info {
    self.0
  }

  unsafe fn ptr_to_str(ptr: *const c_char) -> &'static CUtf8 {
    // Safety: ptr must've come from a &'static CUtf8
    unsafe { CUtf8::from_c_str_unchecked(CStr::from_ptr(ptr)) }
  }
}

impl From<SystemInfo> for retro_system_info {
  fn from(info: SystemInfo) -> Self {
    info.into_inner()
  }
}

pub struct Runtime {
  audio_sample: retro_audio_sample_t,
  audio_sample_batch: retro_audio_sample_batch_t,
  input_state: retro_input_state_t,
  video_refresh: retro_video_refresh_t,
}

impl Runtime {
  pub fn new(
    audio_sample: retro_audio_sample_t,
    audio_sample_batch: retro_audio_sample_batch_t,
    input_state: retro_input_state_t,
    video_refresh: retro_video_refresh_t,
  ) -> Runtime {
    Runtime {
      audio_sample,
      audio_sample_batch,
      input_state,
      video_refresh,
    }
  }

  /// Sends audio data to the `libretro` frontend.
  pub fn upload_audio_frame(&self, frame: &[i16]) -> usize {
    let cb = self
      .audio_sample_batch
      .expect("`upload_audio_frame` called without registering an `audio_sample_batch` callback");

    unsafe { cb(frame.as_ptr(), frame.len() / 2) }
  }

  /// Sends audio data to the `libretro` frontend.
  pub fn upload_audio_sample(&self, left: i16, right: i16) {
    let cb = self
      .audio_sample
      .expect("`upload_audio_sample` called without registering an `audio_sample` callback");

    unsafe { cb(left, right) }
  }

  /// Sends video data to the `libretro` frontend.
  pub fn upload_video_frame(&self, frame: &[u8], width: u32, height: u32, pitch: usize) {
    let cb = self
      .video_refresh
      .expect("`upload_video_frame` called without registering a `video_refresh` callback");

    unsafe { cb(frame.as_ptr() as *const c_void, width, height, pitch) }
  }

  /// Returns true if the specified button is pressed, false otherwise.
  pub fn is_joypad_button_pressed(&self, port: DevicePort, btn: JoypadButton) -> bool {
    let cb = self
      .input_state
      .expect("`is_joypad_button_pressed` called without registering an `input_state` callback");

    let port = c_uint::from(port.into_inner());
    let device = RETRO_DEVICE_JOYPAD;
    let index = 0;
    let id = btn.into();
    unsafe { cb(port, device, index, id) != 0 }
  }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RetroVariable<'a>(pub Option<&'a CStr>);

impl<'a> Deref for RetroVariable<'a> {
  type Target = Option<&'a CStr>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

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
  pub fn on_set_environment(&mut self, mut env: env::EnvironmentPtr) {
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
      let data = ::core::slice::from_raw_parts_mut(data as *mut u8, size);
      let mut env = self.environment();
      self.core_ref(|core| core.serialize(&mut env, data).is_ok())
    }
  }

  /// Invoked by a `libretro` frontend, with the `retro_unserialize` API call.
  pub fn on_unserialize(&mut self, data: *const (), size: usize) -> bool {
    unsafe {
      let data = ::core::slice::from_raw_parts(data as *const u8, size);
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
  fn environment(&self) -> env::EnvironmentPtr {
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
      use ::core::ffi::c_char;
      use ::core::ffi::*;
      use ::libretro_rs::ffi::*;
      use ::libretro_rs::retro::*;

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
      extern "C" fn retro_set_environment(cb: env::EnvironmentPtr) {
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
