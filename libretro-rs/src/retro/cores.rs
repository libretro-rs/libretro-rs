use crate::ffi::*;
use crate::retro::*;
use core::ffi::*;
use core::ops::*;

#[allow(unused_variables)]
/// A libretro core.
///
/// Each of the methods and associated functions corresponds to a C function in
/// the libretro API, with additional parameters for the frontend-provided callbacks.
/// Some methods return [`Result`] despite the corresponding C function having no
/// return value so that implementers can use the `?` operator.
pub trait Core: Sized {
  /// Called during `retro_set_environment`.
  fn set_environment(env: &mut impl env::SetEnvironment<Self>) {}

  /// Called during `retro_init`. This function is provided for the sake of completeness; it's generally redundant
  /// with [`Core::load_game`].
  fn init(env: &mut impl env::Init<Self>) {}

  /// Called to get information about the core. This information can then be displayed in a frontend, or used to
  /// construct core-specific paths.
  fn get_system_info() -> SystemInfo;

  fn get_system_av_info(&self, env: &mut impl env::GetAvInfo<Self>) -> SystemAVInfo;

  fn get_region(&self, env: &mut impl env::GetRegion<Self>) -> Region {
    Region::NTSC
  }

  /// Called to associate a particular device with a particular port. A core is allowed to ignore this request.
  ///
  /// This function returns [`Result`] to make error handling easier.
  /// The libretro function `retro_set_controller_port_device` does not return a result to the frontend.
  fn set_controller_port_device(
    &mut self,
    env: &mut impl env::SetPortDevice<Self>,
    port: DevicePort,
    device: DeviceTypeId,
  ) -> Result<()> {
    Err(CoreError::new())
  }

  /// Called when a player resets their game.
  fn reset(&mut self, env: &mut impl env::Reset<Self>);

  /// Called continuously once the core is initialized and a game is loaded.
  ///
  /// The core is expected to advance emulation by a single frame before returning.
  /// The core must call [`Runtime::poll_inputs`] at least once.
  fn run(&mut self, env: &mut impl env::Run<Self>, runtime: &mut impl Runtime) -> InputsPolled;

  /// Called to determine the size of the save state buffer. This is only ever called once per run, and the core must
  /// not exceed the size returned here for subsequent saves.
  fn serialize_size(&self, env: &mut impl env::SerializeSize<Self>) -> usize {
    0
  }

  /// Allows a core to save its internal state into the specified buffer. The buffer is guaranteed to be at least `size`
  /// bytes, where `size` is the value returned from `serialize_size`.
  fn serialize(&self, env: &mut impl env::Serialize<Self>, data: &mut [u8]) -> Result<()> {
    Err(CoreError::new())
  }

  /// Allows a core to load its internal state from the specified buffer. The buffer is guaranteed to be at least `size`
  /// bytes, where `size` is the value returned from `serialize_size`.
  fn unserialize(&mut self, env: &mut impl env::Unserialize<Self>, data: &[u8]) -> Result<()> {
    Err(CoreError::new())
  }

  fn cheat_reset(&mut self, env: &mut impl env::CheatReset<Self>) {}

  /// Called when a user attempts to apply or remove a cheat code.
  ///
  /// This function returns [`Result`] to make error handling easier.
  /// The libretro function `retro_cheat_set` does not return a result to the frontend.
  fn cheat_set(&mut self, env: &mut impl env::CheatSet<Self>, index: c_uint, enabled: bool, code: &CStr) -> Result<()> {
    Err(CoreError::new())
  }

  /// Called when a new instance of the core is needed. The `env` parameter can be used to set-up and/or query values
  /// required for the core to function properly.
  fn load_game(env: &mut impl env::LoadGame<Self>, game: Game) -> Result<Self>;

  fn load_game_special(env: &mut impl env::LoadGameSpecial<Self>, game_type: GameType, info: Game) -> Result<Self> {
    Err(CoreError::new())
  }

  fn unload_game(&mut self, env: &mut impl env::UnloadGame<Self>) {}

  fn get_memory_size(&self, env: &mut impl env::GetMemorySize<Self>, id: MemoryType) -> Result<usize> {
    Err(CoreError::new())
  }

  fn get_memory_data(&self, env: &mut impl env::GetMemoryData<Self>, id: MemoryType) -> Result<&mut [u8]> {
    Err(CoreError::new())
  }
}

pub unsafe trait GLRenderingCore: Core {
  fn context_reset(&mut self, callbacks: GLContextCallbacks);

  fn context_destroy(&mut self) {}
}

pub unsafe trait GLContext<C: GLRenderingCore>: Sized {
  unsafe fn create(callbacks: GLContextCallbacks, core: &mut C) -> Self;

  unsafe fn reset(&mut self, callbacks: GLContextCallbacks, core: &mut C) {
    *self = Self::create(callbacks, core);
  }

  unsafe fn destroy(&mut self) {}
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
  fn use_hardware_frame_buffer(&mut self, render_callbacks: &impl HWRenderEnabled, width: c_uint, height: c_uint);

  /// Polls all input devices.
  /// Must be called at least once on every call to [`Environment::run`]
  fn poll_inputs(&mut self) -> InputsPolled;

  /// Returns true if the specified button is pressed, false otherwise.
  fn is_joypad_button_pressed(&self, port: DevicePort, btn: JoypadButton) -> bool;
}

pub trait Deinit {
  fn deinit(&mut self);
}

impl Runtime for Callbacks {
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

  fn use_hardware_frame_buffer(&mut self, render_callbacks: &impl HWRenderEnabled, width: c_uint, height: c_uint) {
    unsafe { self.use_hardware_frame_buffer(render_callbacks, width, height) }
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

/// This is the glue layer between a [Core] and the `libretro` API.
#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Instance<C, G> {
  environment: InstanceEnvironment<C, G>,
  system: Option<C>,
  callbacks: Callbacks,
}

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Callbacks {
  audio_sample: retro_audio_sample_t,
  audio_sample_batch: retro_audio_sample_batch_t,
  input_poll: retro_input_poll_t,
  input_state: retro_input_state_t,
  video_refresh: retro_video_refresh_t,
}

impl Callbacks {
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

  unsafe fn use_hardware_frame_buffer(&mut self, _render_callbacks: &impl HWRenderEnabled, width: c_uint, height: c_uint) {
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

impl<C> Instance<C, NoGLData>
where
  C: Core,
  Self: ValidInstance,
{
  pub const fn new(context_reset: non_null_retro_hw_context_reset_t, context_destroy: non_null_retro_hw_context_reset_t) -> Self {
    Self {
      environment: InstanceEnvironment {
        cb: None,
        gl: NoGLData::new(context_reset, context_destroy),
        phantom: core::marker::PhantomData,
      },
      callbacks: Callbacks::new(),
      system: None,
    }
  }
}

impl<C> Instance<C, InstanceGLData>
where
  C: GLRenderingCore,
  Self: ValidInstance,
{
  pub const fn new(context_reset: non_null_retro_hw_context_reset_t, context_destroy: non_null_retro_hw_context_reset_t) -> Self {
    Self {
      environment: InstanceEnvironment {
        cb: None,
        gl: InstanceGLData {
          context_reset,
          context_destroy,
          core_callbacks: None,
        },
        phantom: core::marker::PhantomData,
      },
      system: None,
      callbacks: Callbacks::new(),
    }
  }
}

impl<C: Core, G> Instance<C, G> {
  /// Invoked by a `libretro` frontend, with the `retro_get_system_info` API call.
  pub fn on_get_system_info(&mut self, info: &mut retro_system_info) {
    *info = C::get_system_info().into();
  }

  /// Invoked by a `libretro` frontend, with the `retro_get_system_av_info` API call.
  pub unsafe fn on_get_system_av_info(&mut self, info: &mut retro_system_av_info) {
    let env = &mut self.environment;
    let system = self.system.as_mut().unwrap_unchecked();
    *info = system.get_system_av_info(env).into();
  }

  /// Invoked by a `libretro` frontend, with the `retro_init` API call.
  pub unsafe fn on_init(&mut self) {
    C::init(&mut self.environment);
  }

  /// Invoked by a `libretro` frontend, with the `retro_deinit` API call.
  pub fn on_deinit(&mut self)
  where
    G: Deinit,
  {
    self.system = None;
    self.callbacks = Callbacks::new();
    self.environment.deinit();
  }

  /// Invoked by a `libretro` frontend, with the `retro_set_environment` API call.
  pub fn on_set_environment(&mut self, env: non_null_retro_environment_t) {
    self.environment.cb = Some(env);
    C::set_environment(&mut self.environment);
  }

  /// Invoked by a `libretro` frontend, with the `retro_set_audio_sample` API call.
  pub fn on_set_audio_sample(&mut self, cb: non_null_retro_audio_sample_t) {
    self.callbacks.audio_sample = Some(cb);
  }

  /// Invoked by a `libretro` frontend, with the `retro_set_audio_sample_batch` API call.
  pub fn on_set_audio_sample_batch(&mut self, cb: non_null_retro_audio_sample_batch_t) {
    self.callbacks.audio_sample_batch = Some(cb);
  }

  /// Invoked by a `libretro` frontend, with the `retro_set_input_poll` API call.
  pub fn on_set_input_poll(&mut self, cb: non_null_retro_input_poll_t) {
    self.callbacks.input_poll = Some(cb);
  }

  /// Invoked by a `libretro` frontend, with the `retro_set_input_state` API call.
  pub fn on_set_input_state(&mut self, cb: non_null_retro_input_state_t) {
    self.callbacks.input_state = Some(cb);
  }

  /// Invoked by a `libretro` frontend, with the `retro_set_video_refresh` API call.
  pub fn on_set_video_refresh(&mut self, cb: non_null_retro_video_refresh_t) {
    self.callbacks.video_refresh = Some(cb);
  }

  /// Invoked by a `libretro` frontend, with the `retro_set_controller_port_device` API call.
  pub unsafe fn on_set_controller_port_device(&mut self, port: DevicePort, device: DeviceTypeId) {
    let core = self.system.as_mut().unwrap_unchecked();
    let env = &mut self.environment;
    let _ = core.set_controller_port_device(env, port, device);
  }

  /// Invoked by a `libretro` frontend, with the `retro_reset` API call.
  pub unsafe fn on_reset(&mut self) {
    self.system.as_mut().unwrap_unchecked().reset(&mut self.environment)
  }

  /// Invoked by a `libretro` frontend, with the `retro_run` API call.
  pub unsafe fn on_run(&mut self)
  where
    InstanceEnvironment<C, G>: env::Run<C>,
  {
    let env = &mut self.environment;
    let callbacks = &mut self.callbacks;
    self.system.as_mut().unwrap_unchecked().run(env, callbacks);
  }

  /// Invoked by a `libretro` frontend, with the `retro_serialize_size` API call.
  pub unsafe fn on_serialize_size(&mut self) -> usize {
    let env = &mut self.environment;
    self.system.as_mut().unwrap_unchecked().serialize_size(env)
  }

  /// Invoked by a `libretro` frontend, with the `retro_serialize` API call.
  pub unsafe fn on_serialize(&mut self, data: *mut (), size: usize) -> bool {
    let data = core::slice::from_raw_parts_mut(data as *mut u8, size);
    let env = &mut self.environment;
    self.system.as_mut().unwrap_unchecked().serialize(env, data).is_ok()
  }

  /// Invoked by a `libretro` frontend, with the `retro_unserialize` API call.
  pub unsafe fn on_unserialize(&mut self, data: *const (), size: usize) -> bool {
    let data = core::slice::from_raw_parts(data as *const u8, size);
    let env = &mut self.environment;
    self.system.as_mut().unwrap_unchecked().unserialize(env, data).is_ok()
  }

  /// Invoked by a `libretro` frontend, with the `retro_cheat_reset` API call.
  pub unsafe fn on_cheat_reset(&mut self) {
    let env = &mut self.environment;
    self.system.as_mut().unwrap_unchecked().cheat_reset(env)
  }

  /// Invoked by a `libretro` frontend, with the `retro_cheat_set` API call.
  ///
  /// # Safety
  /// `code` must be a valid argument to [`CStr::from_ptr`].
  pub unsafe fn on_cheat_set(&mut self, index: c_uint, enabled: bool, code: *const c_char) {
    let code = CStr::from_ptr(code);
    let env = &mut self.environment;
    let _ = self.system.as_mut().unwrap_unchecked().cheat_set(env, index, enabled, code);
  }

  /// Invoked by a `libretro` frontend, with the `retro_load_game` API call.
  ///
  /// # Safety
  /// `game` must remain valid until [`Instance::on_unload_game`] is called.
  pub unsafe fn on_load_game(&mut self, game: *const retro_game_info) -> bool
  where
    InstanceEnvironment<C, G>: env::LoadGame<C>,
  {
    let env = &mut self.environment;
    let game = game.as_ref().map_or_else(Game::default, Game::from);
    self.system = C::load_game(env, game).ok();
    self.system.is_some()
  }

  /// Invoked by a `libretro` frontend, with the `retro_load_game_special` API call.
  pub unsafe fn on_load_game_special(&mut self, game_type: GameType, info: &retro_game_info, _num_info: usize) -> bool {
    let env = &mut self.environment;
    self.system = C::load_game_special(env, game_type, info.into()).ok();
    self.system.is_some()
  }

  /// Invoked by a `libretro` frontend, with the `retro_unload_game` API call.
  pub unsafe fn on_unload_game(&mut self) {
    let env = &mut self.environment;
    self.system.as_mut().unwrap_unchecked().unload_game(env)
  }

  /// Invoked by a `libretro` frontend, with the `retro_get_region` API call.
  pub unsafe fn on_get_region(&mut self) -> c_uint {
    let env = &mut self.environment;
    self.system.as_mut().unwrap_unchecked().get_region(env).into()
  }

  /// Invoked by a `libretro` frontend, with the `retro_get_memory_data` API call.
  pub unsafe fn on_get_memory_data(&mut self, id: MemoryType) -> *mut () {
    let env = &mut self.environment;
    self
      .system
      .as_mut()
      .unwrap_unchecked()
      .get_memory_data(env, id)
      .ok()
      .map_or_else(std::ptr::null_mut, |data| data.as_mut_ptr() as *mut ())
  }

  /// Invoked by a `libretro` frontend, with the `retro_get_memory_size` API call.
  pub unsafe fn on_get_memory_size(&mut self, id: MemoryType) -> usize {
    let env = &mut self.environment;
    self.system.as_mut().unwrap_unchecked().get_memory_size(env, id).unwrap_or(0)
  }
}

pub trait ValidInstance {}
impl<C: Core> ValidInstance for Instance<C, NoGLData> {}
impl<C: GLRenderingCore> ValidInstance for Instance<C, InstanceGLData> {}

impl<C: Core> Instance<C, NoGLData> {
  pub fn on_context_destroy(&mut self) {}

  pub fn on_context_reset(&mut self) {}
}

impl<C> Instance<C, InstanceGLData>
where
  C: GLRenderingCore,
  Self: ValidInstance,
{
  pub unsafe fn on_context_destroy(&mut self) {
    self.system.as_mut().unwrap_unchecked().context_destroy();
  }

  pub unsafe fn on_context_reset(&mut self) {
    let callbacks = self.environment.gl.core_callbacks.unwrap_unchecked();
    self.system.as_mut().unwrap_unchecked().context_reset(callbacks);
  }
}

impl<C, G: Deinit> Deinit for InstanceEnvironment<C, G> {
  fn deinit(&mut self) {
    self.cb = None;
    self.gl.deinit();
  }
}

impl<C: Core, G> env::Environment<C> for InstanceEnvironment<C, G> {
  fn get_ptr(&self) -> non_null_retro_environment_t {
    unsafe { self.cb.unwrap_unchecked() }
  }
}

impl<C: Core> env::LoadGame<C> for InstanceEnvironment<C, NoGLData> {
  fn set_hw_render_gl(&mut self, _options: GLOptions) -> env::Result<GLRenderEnabled>
  where
    C: GLRenderingCore,
  {
    Err(CommandError::new())
  }
}

impl<C: GLRenderingCore> env::LoadGame<C> for InstanceEnvironment<C, InstanceGLData> {
  fn set_hw_render_gl(&mut self, options: GLOptions) -> env::Result<GLRenderEnabled>
  where
    C: GLRenderingCore,
  {
    use crate::retro::env::Environment;
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

#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NoGLData;

impl NoGLData {
  pub const fn new(
    _context_reset: non_null_retro_hw_context_reset_t,
    _context_destroy: non_null_retro_hw_context_reset_t,
  ) -> Self {
    NoGLData
  }
}

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InstanceGLData {
  context_reset: non_null_retro_hw_context_reset_t,
  context_destroy: non_null_retro_hw_context_reset_t,
  core_callbacks: Option<GLContextCallbacks>,
}

impl InstanceGLData {
  pub const fn new(context_reset: non_null_retro_hw_context_reset_t, context_destroy: non_null_retro_hw_context_reset_t) -> Self {
    Self {
      context_reset,
      context_destroy,
      core_callbacks: None,
    }
  }
}

impl Deinit for NoGLData {
  fn deinit(&mut self) {}
}

impl Deinit for InstanceGLData {
  fn deinit(&mut self) {
    self.core_callbacks = None;
  }
}

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InstanceEnvironment<C, G> {
  cb: retro_environment_t,
  gl: G,
  phantom: core::marker::PhantomData<C>,
}

impl<C, G> InstanceEnvironment<C, G> {
  pub fn new(cb: retro_environment_t, gl: G) -> Self {
    Self {
      cb,
      gl,
      phantom: core::marker::PhantomData,
    }
  }
}

#[macro_export]
macro_rules! libretro_core {
  (@Core GLState) => {
    NoGLData
  };
  (@GLCore GLState) => {
    InstanceGLData
  };
  ($core:ty) => {
    libretro_core!($core: Core);
  };
  ($core:ty: $cb:ident) => {
    #[doc(hidden)]
    mod __libretro_rs_gen {
      use core::ffi::c_char;
      use core::ffi::*;
      use libretro_rs::ffi::*;
      use libretro_rs::libretro_core;
      use libretro_rs::retro::*;

  type GLState = libretro_core!(@$cb GLState);

      static mut RETRO_INSTANCE: Instance<$core, GLState> = Instance::<$core, GLState>::new(on_context_reset, on_context_destroy);

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

      // Only used as a function pointer, doesn't need no_mangle
      unsafe extern "C" fn on_context_reset() {
        RETRO_INSTANCE.on_context_reset()
      }

      // Only used as a function pointer, doesn't need no_mangle
      unsafe extern "C" fn on_context_destroy() {
        RETRO_INSTANCE.on_context_destroy()
      }
    }
  };
}
