//! Types and macros for implementing libretro cores.
//!
//! The [`Core`] trait contains the essential libretro API functions;
//! Optional libretro functions are provided in additional traits.
//!
//! The [`libretro_core`] macro takes a type that implements these traits and
//! implements the C functions required by the libretro API.
//!
//! Cores should be well-behaved - they shouldn't panic, crash, or invoke
//! undefined behavior, lest they take the frontend down with it.
//! Likewise, cores shouldn't request a shutdown in lieu of panicking.
//! If a core loads content successfully, it should continue to function until
//! the user unloads it or shuts down the frontend.

use crate::ffi::*;
use crate::retro::env::Environment;
use crate::retro::*;
use core::ffi::*;
use core::mem::MaybeUninit;
use core::ops::*;
use std::mem::ManuallyDrop;

/// A basic libretro core.
pub trait Core: Sized {
  type Init: Sized;

  /// Called to get information about the core. This information can then be displayed in a frontend, or used to
  /// construct core-specific paths.
  fn get_system_info() -> SystemInfo;

  /// Called during `retro_set_environment`.
  #[allow(unused_variables)]
  fn set_environment(env: &mut impl env::SetEnvironment) {}

  /// Called during `retro_init`.
  fn init(env: &mut impl env::Init) -> Self::Init;

  /// Called during `retro_load_game` when a core doesn't require a game's path.
  #[allow(unused_variables)]
  fn load_game(env: &mut impl env::LoadGame, init_state: Self::Init, game: &GameData) -> Result<Self, LoadGameError<Self::Init>> {
    Err(LoadGameError::new(init_state))
  }

  /// Called during `retro_load_game` when a core requires a game's path.
  #[allow(unused_variables)]
  fn load_game_from_path(
    env: &mut impl env::LoadGame,
    init_state: Self::Init,
    game: &GamePath,
  ) -> Result<Self, LoadGameError<Self::Init>> {
    Err(LoadGameError::new(init_state))
  }

  #[allow(unused_variables)]
  fn load_without_content(env: &mut impl env::LoadGame, init_state: Self::Init) -> Result<Self, LoadGameError<Self::Init>> {
    Err(LoadGameError::new(init_state))
  }

  fn get_system_av_info(&self, env: &mut impl env::GetAvInfo) -> SystemAVInfo;

  /// Called continuously once the core is initialized and a game is loaded.
  ///
  /// The core is expected to advance emulation by a single frame before returning.
  /// The core must call [`Callbacks::poll_inputs`] at least once.
  fn run(&mut self, env: &mut impl env::Run, callbacks: &mut impl Callbacks) -> InputsPolled;

  /// Called when a player resets their game.
  fn reset(&mut self, env: &mut impl env::Reset);

  /// Called during `retro_unload_game`.
  ///
  /// This will be called before either `retro_deinit` or `retro_load_game`
  #[allow(unused_variables)]
  fn unload_game(self, env: &mut impl env::UnloadGame) -> Self::Init;

  /// Called during `retro_deinit`
  #[allow(unused_variables)]
  fn deinit(env: &mut impl env::Deinit, init_state: Self::Init) {}
}

/// Save state functions.
pub trait SaveStateCore: Core {
  /// Called to determine the size of the save state buffer. This is only ever called once per run, and the core must
  /// not exceed the size returned here for subsequent saves.
  fn serialize_size(&self, env: &mut impl env::SerializeSize) -> core::num::NonZeroUsize;

  /// Allows a core to save its internal state into the specified buffer. The buffer is guaranteed to be at least `size`
  /// bytes, where `size` is the value returned from `serialize_size`.
  fn serialize(&self, env: &mut impl env::Serialize, data: &mut [u8]) -> Result<(), CoreError>;

  /// Allows a core to load its internal state from the specified buffer. The buffer is guaranteed to be at least `size`
  /// bytes, where `size` is the value returned from `serialize_size`.
  fn unserialize(&mut self, env: &mut impl env::Unserialize, data: &[u8]) -> Result<(), CoreError>;
}

/// Implementation of `retro_set_controller_port_device`.
pub trait DeviceTypeAwareCore: Core {
  /// Called to associate a particular device with a particular port. A core is allowed to ignore this request.
  ///
  /// This function returns [`Result`] to make error handling easier.
  /// The libretro function `retro_set_controller_port_device` does not return a result to the frontend.
  fn set_controller_port_device(
    &mut self,
    env: &mut impl env::SetPortDevice,
    port: DevicePort,
    device: DeviceTypeId,
  ) -> Result<(), CoreError>;
}

/// Cheat code functions.
pub trait CheatsCore: Core {
  /// Called when a user attempts to apply or remove a cheat code.
  ///
  /// This function returns [`Result`] to make error handling easier.
  /// The libretro function `retro_cheat_set` does not return a result to the frontend.
  fn cheat_set(&mut self, env: &mut impl env::CheatSet, index: c_uint, enabled: bool, code: &CStr) -> Result<(), CoreError>;

  fn cheat_reset(&mut self, env: &mut impl env::CheatReset);
}

/// Functions for getting memory regions (e.g. save RAM.)
pub trait GetMemoryRegionCore: Core {
  fn get_memory_size(&self, env: &mut impl env::GetMemorySize, id: MemoryType) -> usize;

  fn get_memory_data(&self, env: &mut impl env::GetMemoryData, id: MemoryType) -> Option<&mut [u8]>;
}

/// Implementation of `retro_load_game` for content-less cores.
pub trait NoGameCore: Core {
  fn load_game(env: &mut impl env::LoadGame, init_state: Self::Init) -> Result<Self, LoadGameError<Self::Init>>;
}

/// Implementation of `retro_load_game_special`. Should be avoided if possible.
pub trait SpecialGameCore: Core {
  fn load_game_from_path(
    env: &mut impl env::LoadGameSpecial,
    init_state: Self::Init,
    game_type: GameType,
    games: &[SpecialGamePath],
  ) -> Result<Self, LoadGameError<Self::Init>>;

  fn load_game(
    env: &mut impl env::LoadGameSpecial,
    init_state: Self::Init,
    game_type: GameType,
    games: &[SpecialGameData],
  ) -> Result<Self, LoadGameError<Self::Init>>;
}

/// Implementation of `retro_get_region`.
///
/// This is vestigial functionality; RetroArch no longer calls this function.
/// If a core does not implement this trait, the [`libretro_core`] macro will
/// return [`RETRO_REGION_NTSC`], which is the de facto default value.
pub trait RegionAwareCore: Core {
  fn get_region(&self, env: &mut impl env::GetRegion) -> Region;
}

/// OpenGL context management functions.
pub unsafe trait OpenGLCore: Core {
  fn context_reset(&mut self, env: &mut impl Environment, callbacks: GLContextCallbacks);

  fn context_destroy(&mut self, env: &mut impl Environment);
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

impl AsRef<retro_system_info> for SystemInfo {
  fn as_ref(&self) -> &retro_system_info {
    &self.0
  }
}

impl From<SystemInfo> for retro_system_info {
  fn from(info: SystemInfo) -> Self {
    info.into_inner()
  }
}

pub trait Callbacks {
  /// Sends audio data to the `libretro` frontend.
  fn upload_audio_frame(&mut self, frame: &[i16]) -> usize;

  /// Sends audio data to the `libretro` frontend.
  fn upload_audio_sample(&mut self, left: i16, right: i16);

  /// Sends video data to the `libretro` frontend.
  /// Must not be called if hardware rendering is used;
  /// call `use_hardware_frame_buffer` instead.
  fn upload_video_frame(&mut self, frame: &[u8], width: c_uint, height: c_uint, pitch: usize);

  /// Explicitly informs the `libretro` frontend to repeat the previous video frame.
  /// Must only be called if [`Environment::get_can_dupe`] returns `true`.
  fn repeat_video_frame(&mut self);

  /// When using hardware rendering, informs the `libretro` frontend that core
  /// has finished rendering to the frame buffer.
  fn use_hardware_frame_buffer(&mut self, hw_render_enabled: &impl HWRenderEnabled, width: c_uint, height: c_uint);

  /// Polls all input devices.
  /// Must be called at least once on every call to [`Environment::run`]
  fn poll_inputs(&mut self) -> InputsPolled;

  /// Returns true if the specified button is pressed, false otherwise.
  fn is_joypad_button_pressed(&self, port: DevicePort, btn: JoypadButton) -> bool;
}

impl Callbacks for InstanceCallbacks {
  fn upload_audio_frame(&mut self, frame: &[i16]) -> usize {
    unsafe { self.upload_audio_frame(frame) }
  }

  fn upload_audio_sample(&mut self, left: i16, right: i16) {
    unsafe { self.upload_audio_sample(left, right) }
  }

  fn upload_video_frame(&mut self, frame: &[u8], width: c_uint, height: c_uint, pitch: usize) {
    unsafe { self.upload_video_frame(frame, width, height, pitch) }
  }

  fn repeat_video_frame(&mut self) {
    unsafe { self.repeat_video_frame() }
  }

  fn use_hardware_frame_buffer(&mut self, hw_render_enabled: &impl HWRenderEnabled, width: c_uint, height: c_uint) {
    unsafe { self.use_hardware_frame_buffer(hw_render_enabled, width, height) }
  }

  fn poll_inputs(&mut self) -> InputsPolled {
    unsafe { self.poll_inputs() }
  }

  /// Returns true if the specified button is pressed, false otherwise.
  fn is_joypad_button_pressed(&self, port: DevicePort, btn: JoypadButton) -> bool {
    unsafe { self.is_joypad_button_pressed(port, btn) }
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

/// This is the glue layer between a [`Core`] and the `libretro` API.
#[doc(hidden)]
#[derive(Debug)]
pub struct Instance<I, C> {
  env: InstanceEnvironment,
  cb: InstanceCallbacks,
  core: MaybeUninit<CoreState<I, C>>,
}

impl<I, C> Instance<I, C> {
  pub const fn new(context_reset: non_null_retro_hw_context_reset_t, context_destroy: non_null_retro_hw_context_reset_t) -> Self {
    Self {
      env: InstanceEnvironment {
        cb: None,
        gl: InstanceGLState::new(context_reset, context_destroy),
      },
      cb: InstanceCallbacks::new(),
      core: MaybeUninit::uninit(),
    }
  }

  pub fn on_set_audio_sample(&mut self, cb: non_null_retro_audio_sample_t) {
    self.cb.audio_sample = Some(cb);
  }

  pub fn on_set_audio_sample_batch(&mut self, cb: non_null_retro_audio_sample_batch_t) {
    self.cb.audio_sample_batch = Some(cb);
  }

  pub fn on_set_input_poll(&mut self, cb: non_null_retro_input_poll_t) {
    self.cb.input_poll = Some(cb);
  }

  pub fn on_set_input_state(&mut self, cb: non_null_retro_input_state_t) {
    self.cb.input_state = Some(cb);
  }

  pub fn on_set_video_refresh(&mut self, cb: non_null_retro_video_refresh_t) {
    self.cb.video_refresh = Some(cb);
  }
}

// The following code exploits the fact that inherent impls can shadow trait
// impls. The inherent impl provides a functioning implementation when the core
// implements specific traits, and the trait impl provides default methods for
// when it doesn't. Either way, the libretro_core macro can call the method
// using the same syntax and without the need to disambiguate the call.

impl<C: Core> Instance<C::Init, C> {
  pub fn on_get_system_info(&mut self, info: &mut retro_system_info) {
    *info = C::get_system_info().into()
  }

  pub fn on_set_environment(&mut self, env: non_null_retro_environment_t) {
    self.env.cb = Some(env);
    C::set_environment(&mut self.env);
  }

  pub unsafe fn on_init(&mut self) {
    self.core.write(CoreState::init(C::init(&mut self.env)));
  }

  pub unsafe fn on_load_game(&mut self, game: *const retro_game_info) -> bool {
    let Instance { env, core, .. } = self;
    let init = take_init(core);
    let result = match game.as_ref() {
      Some(game) => {
        let ptr: *const retro_game_info = game;
        if game.data.is_null() {
          C::load_game_from_path(env, init, &*(ptr.cast()))
        } else {
          C::load_game(env, init, &*(ptr.cast()))
        }
      }
      None => C::load_without_content(env, init),
    };
    update_system(core, result)
  }

  pub unsafe fn on_get_system_av_info(&mut self, info: &mut retro_system_av_info) {
    let Instance { env, core, .. } = self;
    *info = loaded_mut(core).get_system_av_info(env).into();
  }

  pub unsafe fn on_run(&mut self) {
    loaded_mut(&mut self.core).run(&mut self.env, &mut self.cb);
  }

  pub unsafe fn on_reset(&mut self) {
    loaded_mut(&mut self.core).reset(&mut self.env);
  }

  pub unsafe fn on_unload_game(&mut self) {
    let Instance { env, core, .. } = self;
    *self.core.assume_init_mut().init = take_loaded(core).unload_game(env);
  }

  pub unsafe fn on_deinit(&mut self) {
    C::deinit(&mut self.env, take_init(&mut self.core));
  }
}

impl<C: SaveStateCore> Instance<C::Init, C> {
  /// Invoked by a `libretro` frontend, with the `retro_serialize_size` API call.
  pub unsafe fn on_serialize_size(&mut self) -> usize {
    loaded_mut(&mut self.core).serialize_size(&mut self.env).get()
  }

  /// Invoked by a `libretro` frontend, with the `retro_serialize` API call.
  pub unsafe fn on_serialize(&mut self, data: *mut (), size: usize) -> bool {
    let data = core::slice::from_raw_parts_mut(data as *mut u8, size);
    loaded_mut(&mut self.core).serialize(&mut self.env, data).is_ok()
  }

  /// Invoked by a `libretro` frontend, with the `retro_unserialize` API call.
  pub unsafe fn on_unserialize(&mut self, data: *const (), size: usize) -> bool {
    let data = core::slice::from_raw_parts(data as *const u8, size);
    loaded_mut(&mut self.core).unserialize(&mut self.env, data).is_ok()
  }
}

#[doc(hidden)]
pub trait SaveStateCoreFallbacks {
  unsafe fn on_serialize_size(&mut self) -> usize {
    0
  }

  unsafe fn on_serialize(&mut self, _data: *mut (), _size: usize) -> bool {
    false
  }

  unsafe fn on_unserialize(&mut self, _data: *const (), _size: usize) -> bool {
    false
  }
}
impl<I, C> SaveStateCoreFallbacks for Instance<I, C> {}

impl<C: DeviceTypeAwareCore> Instance<C::Init, C> {
  /// Invoked by a `libretro` frontend, with the `retro_set_controller_port_device` API call.
  pub unsafe fn on_set_controller_port_device(&mut self, port: DevicePort, device: DeviceTypeId) {
    let system = loaded_mut(&mut self.core);
    let env = &mut self.env;
    let _ = system.set_controller_port_device(env, port, device);
  }
}

#[doc(hidden)]
pub trait DeviceTypeAwareCoreFallbacks {
  unsafe fn on_set_controller_port_device(&mut self, _port: DevicePort, _device: DeviceTypeId) {}
}
impl<I, C> DeviceTypeAwareCoreFallbacks for Instance<I, C> {}

impl<C: CheatsCore> Instance<C::Init, C> {
  /// Invoked by a `libretro` frontend, with the `retro_cheat_set` API call.
  ///
  /// # Safety
  /// `code` must be a valid argument to [`CStr::from_ptr`].
  pub unsafe fn on_cheat_set(&mut self, index: c_uint, enabled: bool, code: *const c_char) {
    let code = CStr::from_ptr(code);
    let env = &mut self.env;
    let _ = loaded_mut(&mut self.core).cheat_set(env, index, enabled, code);
  }

  /// Invoked by a `libretro` frontend, with the `retro_cheat_reset` API call.
  pub unsafe fn on_cheat_reset(&mut self) {
    loaded_mut(&mut self.core).cheat_reset(&mut self.env)
  }
}

#[doc(hidden)]
pub trait CheatsCoreFallbacks {
  unsafe fn on_cheat_set(&mut self, _index: c_uint, _enabled: bool, _code: *const c_char) {}

  unsafe fn on_cheat_reset(&mut self) {}
}
impl<I, C> CheatsCoreFallbacks for Instance<I, C> {}

impl<C: GetMemoryRegionCore> Instance<C::Init, C> {
  /// Invoked by a `libretro` frontend, with the `retro_get_memory_data` API call.
  pub unsafe fn on_get_memory_data(&mut self, id: MemoryType) -> *mut () {
    loaded_mut(&mut self.core)
      .get_memory_data(&mut self.env, id)
      .map_or_else(std::ptr::null_mut, |data| data.as_mut_ptr() as *mut ())
  }

  /// Invoked by a `libretro` frontend, with the `retro_get_memory_size` API call.
  pub unsafe fn on_get_memory_size(&mut self, id: MemoryType) -> usize {
    loaded_mut(&mut self.core).get_memory_size(&mut self.env, id)
  }
}

#[doc(hidden)]
pub trait GetMemoryRegionCoreFallbacks {
  unsafe fn on_get_memory_data(&mut self, _id: MemoryType) -> *mut () {
    core::ptr::null_mut()
  }

  unsafe fn on_get_memory_size(&mut self, _id: MemoryType) -> usize {
    0
  }
}
impl<I, C> GetMemoryRegionCoreFallbacks for Instance<I, C> {}

impl<C: SpecialGameCore> Instance<C::Init, C> {
  /// Invoked by a `libretro` frontend, with the `retro_load_game_special` API call.
  pub unsafe fn on_load_game_special(&mut self, game_type: GameType, info: &retro_game_info, num_info: usize) -> bool {
    let Instance { env, core, .. } = self;
    let init = take_init(core);
    let ptr: *const retro_game_info = info;
    let result = if C::get_system_info().need_fullpath() {
      let games = core::slice::from_raw_parts(ptr.cast(), num_info);
      <C as SpecialGameCore>::load_game_from_path(env, init, game_type, games)
    } else {
      let games = core::slice::from_raw_parts(ptr.cast(), num_info);
      <C as SpecialGameCore>::load_game(env, init, game_type, games)
    };
    update_system(&mut self.core, result)
  }
}

#[doc(hidden)]
pub trait SpecialGameCoreFallbacks {
  unsafe fn on_load_game_special(&mut self, _game_type: GameType, _info: &retro_game_info, _num_info: usize) -> bool {
    false
  }
}
impl<I, C> SpecialGameCoreFallbacks for Instance<I, C> {}

impl<C: RegionAwareCore> Instance<C::Init, C> {
  /// Invoked by a `libretro` frontend, with the `retro_get_region` API call.
  pub unsafe fn on_get_region(&mut self) -> c_uint {
    let env = &mut self.env;
    loaded_mut(&mut self.core).get_region(env).into()
  }
}

#[doc(hidden)]
pub trait RegionAwareCoreFallbacks {
  unsafe fn on_get_region(&mut self) -> c_uint {
    RETRO_REGION_NTSC.into()
  }
}
impl<I, C> RegionAwareCoreFallbacks for Instance<I, C> {}

impl<C: OpenGLCore> Instance<C::Init, C> {
  pub unsafe fn on_context_reset(&mut self) {
    let callbacks = self.env.gl.core_callbacks.unwrap_unchecked();
    loaded_mut(&mut self.core).context_reset(&mut self.env, callbacks);
  }

  pub unsafe fn on_context_destroy(&mut self) {
    loaded_mut(&mut self.core).context_destroy(&mut self.env);
  }
}

#[doc(hidden)]
pub trait OpenGLCoreFallbacks {
  unsafe fn on_context_reset(&mut self) {}

  unsafe fn on_context_destroy(&mut self) {}
}
impl<I, C> OpenGLCoreFallbacks for Instance<I, C> {}

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InstanceEnvironment {
  cb: retro_environment_t,
  gl: InstanceGLState,
}

impl InstanceEnvironment {
  pub const fn new(cb: retro_environment_t, gl: InstanceGLState) -> Self {
    Self { cb, gl }
  }
}

impl Environment for InstanceEnvironment {
  fn get_ptr(&self) -> non_null_retro_environment_t {
    unsafe { self.cb.unwrap_unchecked() }
  }
}

impl env::LoadGame for InstanceEnvironment {
  fn set_hw_render_none(&mut self) -> env::Result<()> {
    let data = retro_hw_render_callback::default();
    unsafe { self.cmd(RETRO_ENVIRONMENT_SET_HW_RENDER, data) }.map(|_: retro_hw_render_callback| ())
  }

  fn set_hw_render_gl(&mut self, options: GLOptions) -> env::Result<GLRenderEnabled> {
    let mut data: retro_hw_render_callback = options.into();
    data.context_destroy = Some(self.gl.context_destroy);
    data.context_reset = Some(self.gl.context_reset);
    unsafe {
      let data: retro_hw_render_callback = self.cmd(RETRO_ENVIRONMENT_SET_HW_RENDER, data)?;
      self.gl.core_callbacks = Some(GLContextCallbacks {
        get_current_framebuffer_cb: data.get_current_framebuffer.unwrap_unchecked(),
        get_proc_address_cb: data.get_proc_address.unwrap_unchecked(),
      });
    }
    Ok(GLRenderEnabled(()))
  }
}

unsafe fn take_init<I, C>(core: &mut MaybeUninit<CoreState<I, C>>) -> I {
  ManuallyDrop::take(&mut core.assume_init_mut().init)
}

unsafe fn loaded_mut<I, C>(core: &mut MaybeUninit<CoreState<I, C>>) -> &mut C {
  core.assume_init_mut().loaded.deref_mut()
}

unsafe fn take_loaded<I, C>(core: &mut MaybeUninit<CoreState<I, C>>) -> C {
  ManuallyDrop::take(&mut core.assume_init_mut().loaded)
}

unsafe fn update_system<C: Core>(
  core: &mut MaybeUninit<CoreState<C::Init, C>>,
  load_game_result: Result<C, LoadGameError<C::Init>>,
) -> bool {
  match load_game_result {
    Ok(loaded) => {
      core.write(CoreState::loaded(loaded));
      true
    }
    Err(err) => {
      core.write(CoreState::init(err.into_inner()));
      false
    }
  }
}

#[doc(hidden)]
pub union CoreState<I, C> {
  init: ManuallyDrop<I>,
  loaded: ManuallyDrop<C>,
}

impl<I, C> CoreState<I, C> {
  fn init(init_state: I) -> Self {
    Self {
      init: ManuallyDrop::new(init_state),
    }
  }

  fn loaded(loaded: C) -> Self {
    Self {
      loaded: ManuallyDrop::new(loaded),
    }
  }
}

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InstanceCallbacks {
  audio_sample: retro_audio_sample_t,
  audio_sample_batch: retro_audio_sample_batch_t,
  input_poll: retro_input_poll_t,
  input_state: retro_input_state_t,
  video_refresh: retro_video_refresh_t,
}

impl InstanceCallbacks {
  pub const fn new() -> Self {
    Self {
      audio_sample: None,
      audio_sample_batch: None,
      input_poll: None,
      input_state: None,
      video_refresh: None,
    }
  }

  unsafe fn upload_audio_frame(&mut self, frame: &[i16]) -> usize {
    self.audio_sample_batch.unwrap_unchecked()(frame.as_ptr(), frame.len() / 2)
  }

  unsafe fn upload_audio_sample(&mut self, left: i16, right: i16) {
    self.audio_sample.unwrap_unchecked()(left, right)
  }

  unsafe fn upload_video_frame(&mut self, frame: &[u8], width: c_uint, height: c_uint, pitch: usize) {
    self.video_refresh.unwrap_unchecked()(frame.as_ptr() as *const c_void, width, height, pitch)
  }

  unsafe fn repeat_video_frame(&mut self) {
    self.video_refresh.unwrap_unchecked()(core::ptr::null(), 0, 0, 0)
  }

  unsafe fn use_hardware_frame_buffer(&mut self, _hw_render_enabled: &impl HWRenderEnabled, width: c_uint, height: c_uint) {
    self.video_refresh.unwrap_unchecked()(RETRO_HW_FRAME_BUFFER_VALID, width, height, 0)
  }

  unsafe fn poll_inputs(&mut self) -> InputsPolled {
    self.input_poll.unwrap_unchecked()();
    InputsPolled(())
  }

  /// Returns true if the specified button is pressed, false otherwise.
  unsafe fn is_joypad_button_pressed(&self, port: DevicePort, btn: JoypadButton) -> bool {
    let port = c_uint::from(port.into_inner());
    let device = RETRO_DEVICE_JOYPAD;
    let index = 0;
    let id = btn.into();
    self.input_state.unwrap_unchecked()(port, device, index, id) != 0
  }
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InstanceGLState {
  context_reset: non_null_retro_hw_context_reset_t,
  context_destroy: non_null_retro_hw_context_reset_t,
  core_callbacks: Option<GLContextCallbacks>,
}

impl InstanceGLState {
  pub const fn new(context_reset: non_null_retro_hw_context_reset_t, context_destroy: non_null_retro_hw_context_reset_t) -> Self {
    Self {
      context_reset,
      context_destroy,
      core_callbacks: None,
    }
  }
}

#[macro_export]
macro_rules! libretro_core {
  ($core:ty) => {
    #[doc(hidden)]
    mod __libretro_rs_gen {
      use core::ffi::c_char;
      use core::ffi::*;
      use libretro_rs::ffi::*;
      use libretro_rs::libretro_core;
      use libretro_rs::retro::*;

      static mut RETRO_INSTANCE: Instance<<$core as Core>::Init, $core> = Instance::new(on_context_reset, on_context_destroy);

      #[no_mangle]
      extern "C" fn retro_api_version() -> c_uint {
        RETRO_API_VERSION
      }

      #[no_mangle]
      unsafe extern "C" fn retro_get_system_info(info: &mut retro_system_info) {
        RETRO_INSTANCE.on_get_system_info(info)
      }

      #[no_mangle]
      unsafe extern "C" fn retro_get_system_av_info(info: &mut retro_system_av_info) {
        RETRO_INSTANCE.on_get_system_av_info(info)
      }

      #[no_mangle]
      unsafe extern "C" fn retro_init() {
        RETRO_INSTANCE.on_init()
      }

      #[no_mangle]
      unsafe extern "C" fn retro_deinit() {
        RETRO_INSTANCE.on_deinit()
      }

      #[no_mangle]
      unsafe extern "C" fn retro_set_environment(cb: non_null_retro_environment_t) {
        RETRO_INSTANCE.on_set_environment(cb)
      }

      #[no_mangle]
      unsafe extern "C" fn retro_set_audio_sample(cb: non_null_retro_audio_sample_t) {
        RETRO_INSTANCE.on_set_audio_sample(cb)
      }

      #[no_mangle]
      unsafe extern "C" fn retro_set_audio_sample_batch(cb: non_null_retro_audio_sample_batch_t) {
        RETRO_INSTANCE.on_set_audio_sample_batch(cb)
      }

      #[no_mangle]
      unsafe extern "C" fn retro_set_input_poll(cb: non_null_retro_input_poll_t) {
        RETRO_INSTANCE.on_set_input_poll(cb)
      }

      #[no_mangle]
      unsafe extern "C" fn retro_set_input_state(cb: non_null_retro_input_state_t) {
        RETRO_INSTANCE.on_set_input_state(cb)
      }

      #[no_mangle]
      unsafe extern "C" fn retro_set_video_refresh(cb: non_null_retro_video_refresh_t) {
        RETRO_INSTANCE.on_set_video_refresh(cb)
      }

      #[no_mangle]
      unsafe extern "C" fn retro_set_controller_port_device(port: DevicePort, device: DeviceTypeId) {
        RETRO_INSTANCE.on_set_controller_port_device(port, device)
      }

      #[no_mangle]
      unsafe extern "C" fn retro_reset() {
        RETRO_INSTANCE.on_reset()
      }

      #[no_mangle]
      unsafe extern "C" fn retro_run() {
        RETRO_INSTANCE.on_run()
      }

      #[no_mangle]
      unsafe extern "C" fn retro_serialize_size() -> usize {
        RETRO_INSTANCE.on_serialize_size()
      }

      #[no_mangle]
      unsafe extern "C" fn retro_serialize(data: *mut (), size: usize) -> bool {
        RETRO_INSTANCE.on_serialize(data, size)
      }

      #[no_mangle]
      unsafe extern "C" fn retro_unserialize(data: *const (), size: usize) -> bool {
        RETRO_INSTANCE.on_unserialize(data, size)
      }

      #[no_mangle]
      unsafe extern "C" fn retro_cheat_reset() {
        RETRO_INSTANCE.on_cheat_reset()
      }

      #[no_mangle]
      unsafe extern "C" fn retro_cheat_set(index: c_uint, enabled: bool, code: *const c_char) {
        RETRO_INSTANCE.on_cheat_set(index, enabled, code)
      }

      #[no_mangle]
      unsafe extern "C" fn retro_load_game(game: *const retro_game_info) -> bool {
        RETRO_INSTANCE.on_load_game(game)
      }

      #[no_mangle]
      unsafe extern "C" fn retro_load_game_special(game_type: GameType, info: &retro_game_info, num_info: usize) -> bool {
        RETRO_INSTANCE.on_load_game_special(game_type, info, num_info)
      }

      #[no_mangle]
      unsafe extern "C" fn retro_unload_game() {
        RETRO_INSTANCE.on_unload_game()
      }

      #[no_mangle]
      unsafe extern "C" fn retro_get_region() -> c_uint {
        RETRO_INSTANCE.on_get_region()
      }

      #[no_mangle]
      unsafe extern "C" fn retro_get_memory_data(id: MemoryType) -> *mut () {
        RETRO_INSTANCE.on_get_memory_data(id)
      }

      #[no_mangle]
      unsafe extern "C" fn retro_get_memory_size(id: MemoryType) -> usize {
        RETRO_INSTANCE.on_get_memory_size(id)
      }

      // These don't need no_mangle; they're only used through pointers
      unsafe extern "C" fn on_context_reset() {
        RETRO_INSTANCE.on_context_reset()
      }

      unsafe extern "C" fn on_context_destroy() {
        RETRO_INSTANCE.on_context_destroy()
      }
    }
  };
}
