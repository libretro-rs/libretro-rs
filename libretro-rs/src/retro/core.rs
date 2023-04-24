use crate::ffi::*;
use crate::retro::*;
use ::core::ffi::*;
use ::core::ops::*;

#[allow(unused_variables)]
/// A libretro core.
///
/// Each of the methods and associated functions corresponds to a C function in
/// the libretro API, with additional parameters for the frontend-provided callbacks.
/// Some methods return [`Result`] despite the corresponding C function having no
/// return value so that implementers can use the `?` operator.
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
  ///
  /// This function returns [`Result`] to make error handling easier.
  /// The libretro function `retro_set_controller_port_device` does not return a result to the frontend.
  fn set_controller_port_device(
    &mut self,
    env: &mut impl env::SetPortDevice,
    port: DevicePort,
    device: DeviceTypeId,
  ) -> Result<()> {
    Err(CoreError::new())
  }

  /// Called when a player resets their game.
  fn reset(&mut self, env: &mut impl env::Reset);

  /// Called continuously once the core is initialized and a game is loaded.
  ///
  /// The core is expected to advance emulation by a single frame before returning.
  /// The core must call [`Runtime::poll_inputs`] at least once.
  fn run(&mut self, env: &mut impl env::Run, runtime: &mut impl Runtime) -> InputsPolled;

  /// Called to determine the size of the save state buffer. This is only ever called once per run, and the core must
  /// not exceed the size returned here for subsequent saves.
  fn serialize_size(&self, env: &mut impl env::SerializeSize) -> usize {
    0
  }

  /// Allows a core to save its internal state into the specified buffer. The buffer is guaranteed to be at least `size`
  /// bytes, where `size` is the value returned from `serialize_size`.
  fn serialize(&self, env: &mut impl env::Serialize, data: &mut [u8]) -> Result<()> {
    Err(CoreError::new())
  }

  /// Allows a core to load its internal state from the specified buffer. The buffer is guaranteed to be at least `size`
  /// bytes, where `size` is the value returned from `serialize_size`.
  fn unserialize(&mut self, env: &mut impl env::Unserialize, data: &[u8]) -> Result<()> {
    Err(CoreError::new())
  }

  fn cheat_reset(&mut self, env: &mut impl env::CheatReset) {}

  /// Called when a user attempts to apply or remove a cheat code.
  ///
  /// This function returns [`Result`] to make error handling easier.
  /// The libretro function `retro_cheat_set` does not return a result to the frontend.
  fn cheat_set(&mut self, env: &mut impl env::CheatSet, index: c_uint, enabled: bool, code: &CStr) -> Result<()> {
    Err(CoreError::new())
  }

  /// Called when a new instance of the core is needed. The `env` parameter can be used to set-up and/or query values
  /// required for the core to function properly.
  fn load_game(env: &mut impl env::LoadGame, game: Game) -> Result<Self>;

  fn load_game_special(env: &mut impl env::LoadGameSpecial, game_type: GameType, info: Game) -> Result<Self> {
    Err(CoreError::new())
  }

  fn unload_game(&mut self, env: &mut impl env::UnloadGame) {}

  fn get_memory_size(&self, env: &mut impl env::GetMemorySize, id: MemoryType) -> Result<usize> {
    Err(CoreError::new())
  }

  fn get_memory_data(&self, env: &mut impl env::GetMemoryData, id: MemoryType) -> Result<&mut [u8]> {
    Err(CoreError::new())
  }
}

/// Rust interface for [`retro_system_info`].
#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct SystemInfo(retro_system_info);

impl SystemInfo {
  /// Minimal constructor. Leaves [`SystemInfo::need_fullpath`] and
  /// [`SystemInfo::block_extract`] set to [false].
  pub fn new<T, U>(library_name: &'static T, library_version: &'static U, valid_extensions: Extensions<'static>) -> Self
  where
    T: AsRef<CStr> + ?Sized,
    U: AsRef<CStr> + ?Sized,
  {
    Self(retro_system_info {
      library_name: library_name.as_ref().as_ptr(),
      library_version: library_version.as_ref().as_ptr(),
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

  pub fn library_name(&self) -> &'static CStr {
    unsafe { CStr::from_ptr(self.0.library_name) }
  }

  pub fn library_version(&self) -> &'static CStr {
    unsafe { CStr::from_ptr(self.0.library_version) }
  }

  pub fn valid_extensions(&self) -> Extensions<'static> {
    Extensions::new(unsafe { CStr::from_ptr(self.0.valid_extensions) })
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
}

impl From<SystemInfo> for retro_system_info {
  fn from(info: SystemInfo) -> Self {
    info.into_inner()
  }
}

pub trait Runtime {
  /// Sends audio data to the `libretro` frontend.
  fn upload_audio_frame(&mut self, frame: &[i16]) -> usize;

  /// Sends audio data to the `libretro` frontend.
  fn upload_audio_sample(&mut self, left: i16, right: i16);

  /// Sends video data to the `libretro` frontend.
  /// Must not be called if hardware rendering is used;
  /// call `use_hardware_frame_buffer` instead.
  fn upload_video_frame(&mut self, frame: &[u8], width: c_uint, height: c_uint, pitch: usize);

  /// Explicitly informs the `libretro` frontend to repeat the previous video frame.
  /// Must only be called if [`env::Environment::get_can_dupe`] returns `true`.
  fn repeat_video_frame(&mut self);

  /// When using hardware rendering, informs the `libretro` frontend that core
  /// has finished rendering to the frame buffer.
  fn use_hardware_frame_buffer(&mut self, width: c_uint, height: c_uint);

  /// Polls all input devices.
  /// Must be called at least once on every call to [`Environment::run`]
  fn poll_inputs(&mut self) -> InputsPolled;

  /// Returns true if the specified button is pressed, false otherwise.
  fn is_joypad_button_pressed(&self, port: DevicePort, btn: JoypadButton) -> bool;
}

#[derive(Clone, Debug)]
pub struct FrontendRuntime {
  audio_sample: non_null_retro_audio_sample_t,
  audio_sample_batch: non_null_retro_audio_sample_batch_t,
  input_poll: non_null_retro_input_poll_t,
  input_state: non_null_retro_input_state_t,
  video_refresh: non_null_retro_video_refresh_t,
}

impl FrontendRuntime {
  pub fn new(
    audio_sample: non_null_retro_audio_sample_t,
    audio_sample_batch: non_null_retro_audio_sample_batch_t,
    input_poll: non_null_retro_input_poll_t,
    input_state: non_null_retro_input_state_t,
    video_refresh: non_null_retro_video_refresh_t,
  ) -> FrontendRuntime {
    FrontendRuntime {
      audio_sample,
      audio_sample_batch,
      input_poll,
      input_state,
      video_refresh,
    }
  }
}

impl Runtime for FrontendRuntime {
  fn upload_audio_frame(&mut self, frame: &[i16]) -> usize {
    unsafe { (self.audio_sample_batch)(frame.as_ptr(), frame.len() / 2) }
  }

  fn upload_audio_sample(&mut self, left: i16, right: i16) {
    unsafe { (self.audio_sample)(left, right) }
  }

  fn upload_video_frame(&mut self, frame: &[u8], width: c_uint, height: c_uint, pitch: usize) {
    unsafe { (self.video_refresh)(frame.as_ptr() as *const c_void, width, height, pitch) }
  }

  fn repeat_video_frame(&mut self) {
    unsafe { (self.video_refresh)(::core::ptr::null(), 0, 0, 0) }
  }

  fn use_hardware_frame_buffer(&mut self, width: c_uint, height: c_uint) {
    unsafe { (self.video_refresh)(RETRO_HW_FRAME_BUFFER_VALID, width, height, 0) }
  }

  fn poll_inputs(&mut self) -> InputsPolled {
    unsafe { (self.input_poll)() };
    InputsPolled(())
  }

  /// Returns true if the specified button is pressed, false otherwise.
  fn is_joypad_button_pressed(&self, port: DevicePort, btn: JoypadButton) -> bool {
    let port = c_uint::from(port.into_inner());
    let device = RETRO_DEVICE_JOYPAD;
    let index = 0;
    let id = btn.into();
    unsafe { (self.input_state)(port, device, index, id) != 0 }
  }
}

pub struct InputsPolled(pub(crate) ());

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RetroVariable<'a>(Option<&'a CStr>);

impl<'a> RetroVariable<'a> {
  pub fn new(str: Option<&'a CStr>) -> Self {
    Self(str)
  }
}

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
  pub environment: Option<env::EnvironmentPtr>,
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
  pub fn on_get_system_av_info(&mut self, info: &mut retro_system_av_info) {
    debug_assert!(
      self.system.is_some(),
      "frontend called retro_get_system_av_info before retro_load_game"
    );
    let env = unsafe { &mut self.environment_cb() };
    let system = unsafe { self.system.as_mut().unwrap_unchecked() };
    *info = system.get_system_av_info(env).into();
  }

  /// Invoked by a `libretro` frontend, with the `retro_init` API call.
  pub fn on_init(&self) {
    T::init(unsafe { &mut self.environment_cb() });
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
  pub fn on_set_controller_port_device(&mut self, port: DevicePort, device: DeviceTypeId) {
    let mut env = unsafe { self.environment_cb() };
    self.core_mut(|core| {
      let _ = core.set_controller_port_device(&mut env, port, device);
    })
  }

  /// Invoked by a `libretro` frontend, with the `retro_reset` API call.
  pub fn on_reset(&mut self) {
    let mut env = unsafe { self.environment_cb() };
    self.core_mut(|core| core.reset(&mut env))
  }

  /// Invoked by a `libretro` frontend, with the `retro_run` API call.
  pub fn on_run(&mut self) {
    let mut env = unsafe { self.environment_cb() };

    let mut runtime = unsafe {
      FrontendRuntime::new(
        self.audio_sample_cb(),
        self.audio_sample_batch_cb(),
        self.input_poll_cb(),
        self.input_state_cb(),
        self.video_refresh_cb(),
      )
    };

    self.core_mut(|core| core.run(&mut env, &mut runtime));
  }

  /// Invoked by a `libretro` frontend, with the `retro_serialize_size` API call.
  pub fn on_serialize_size(&self) -> usize {
    let mut env = unsafe { self.environment_cb() };
    self.core_ref(|core| core.serialize_size(&mut env))
  }

  /// Invoked by a `libretro` frontend, with the `retro_serialize` API call.
  pub fn on_serialize(&self, data: *mut (), size: usize) -> bool {
    unsafe {
      let data = ::core::slice::from_raw_parts_mut(data as *mut u8, size);
      let mut env = self.environment_cb();
      self.core_ref(|core| core.serialize(&mut env, data).is_ok())
    }
  }

  /// Invoked by a `libretro` frontend, with the `retro_unserialize` API call.
  pub fn on_unserialize(&mut self, data: *const (), size: usize) -> bool {
    unsafe {
      let data = ::core::slice::from_raw_parts(data as *const u8, size);
      let mut env = self.environment_cb();
      self.core_mut(|core| core.unserialize(&mut env, data).is_ok())
    }
  }

  /// Invoked by a `libretro` frontend, with the `retro_cheat_reset` API call.
  pub fn on_cheat_reset(&mut self) {
    let mut env = unsafe { self.environment_cb() };
    self.core_mut(|core| core.cheat_reset(&mut env))
  }

  /// Invoked by a `libretro` frontend, with the `retro_cheat_set` API call.
  ///
  /// # Safety
  /// `code` must be a valid argument to [`CStr::from_ptr`].
  pub unsafe fn on_cheat_set(&mut self, index: c_uint, enabled: bool, code: *const c_char) {
    unsafe {
      let code = CStr::from_ptr(code);
      let mut env = self.environment_cb();
      self.core_mut(|core| {
        let _ = core.cheat_set(&mut env, index, enabled, code);
      })
    }
  }

  /// Invoked by a `libretro` frontend, with the `retro_load_game` API call.
  ///
  /// # Safety
  /// `game` must remain valid until [`Instance::on_unload_game`] is called.
  pub unsafe fn on_load_game(&mut self, game: *const retro_game_info) -> bool {
    let mut env = self.environment_cb();
    let game = game.as_ref().map_or_else(Game::default, Game::from);
    self.system = T::load_game(&mut env, game).ok();
    self.system.is_some()
  }

  /// Invoked by a `libretro` frontend, with the `retro_load_game_special` API call.
  pub fn on_load_game_special(&mut self, game_type: GameType, info: &retro_game_info, _num_info: usize) -> bool {
    let mut env = unsafe { self.environment_cb() };
    self.system = T::load_game_special(&mut env, game_type, info.into()).ok();
    self.system.is_some()
  }

  /// Invoked by a `libretro` frontend, with the `retro_unload_game` API call.
  pub fn on_unload_game(&mut self) {
    let mut env = unsafe { self.environment_cb() };
    self.core_mut(|core| core.unload_game(&mut env))
  }

  /// Invoked by a `libretro` frontend, with the `retro_get_region` API call.
  pub fn on_get_region(&self) -> c_uint {
    let mut env = unsafe { self.environment_cb() };
    let system = self.system.as_ref().expect("`on_get_region` called without a game loaded.");
    c_uint::from(system.get_region(&mut env))
  }

  /// Invoked by a `libretro` frontend, with the `retro_get_memory_data` API call.
  pub fn on_get_memory_data(&mut self, id: MemoryType) -> *mut () {
    let mut env = unsafe { self.environment_cb() };
    self.core_ref(|core| {
      core
        .get_memory_data(&mut env, id)
        .ok()
        .map_or_else(std::ptr::null_mut, |data| data.as_mut_ptr() as *mut ())
    })
  }

  /// Invoked by a `libretro` frontend, with the `retro_get_memory_size` API call.
  pub fn on_get_memory_size(&mut self, id: MemoryType) -> usize {
    let mut env = unsafe { self.environment_cb() };
    self.core_ref(|core| core.get_memory_size(&mut env, id).unwrap_or(0))
  }

  #[inline]
  #[doc(hidden)]
  unsafe fn audio_sample_cb(&self) -> non_null_retro_audio_sample_t {
    debug_assert!(
      self.audio_sample.is_some(),
      "Frontend did not set retro_audio_sample_t callback."
    );
    return self.audio_sample.unwrap_unchecked();
  }

  #[inline]
  #[doc(hidden)]
  unsafe fn audio_sample_batch_cb(&self) -> non_null_retro_audio_sample_batch_t {
    debug_assert!(
      self.audio_sample.is_some(),
      "frontend did not set retro_audio_sample_batch_t callback"
    );
    return self.audio_sample_batch.unwrap_unchecked();
  }

  #[inline]
  #[doc(hidden)]
  unsafe fn input_poll_cb(&self) -> non_null_retro_input_poll_t {
    debug_assert!(self.input_poll.is_some(), "frontend did not set retro_input_poll_t callback");
    return self.input_poll.unwrap_unchecked();
  }

  #[inline]
  #[doc(hidden)]
  unsafe fn input_state_cb(&self) -> non_null_retro_input_state_t {
    debug_assert!(
      self.input_state.is_some(),
      "frontend did not set retro_input_state_t callback"
    );
    return self.input_state.unwrap_unchecked();
  }

  #[inline]
  #[doc(hidden)]
  unsafe fn video_refresh_cb(&self) -> non_null_retro_video_refresh_t {
    debug_assert!(
      self.video_refresh.is_some(),
      "frontend did not set retro_video_refresh_t callback"
    );
    return self.video_refresh.unwrap_unchecked();
  }

  #[inline]
  #[doc(hidden)]
  unsafe fn environment_cb(&self) -> env::EnvironmentPtr {
    debug_assert!(
      self.environment.is_some(),
      "frontend did not set retro_environment_t callback"
    );
    self.environment.unwrap_unchecked()
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
        instance_mut(|instance| instance.on_get_system_av_info(info))
      }

      #[no_mangle]
      extern "C" fn retro_init() {
        instance_mut(|instance| instance.on_init())
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
      extern "C" fn retro_set_controller_port_device(port: DevicePort, device: DeviceTypeId) {
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
