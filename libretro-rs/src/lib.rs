pub use libretro_rs_sys as sys;

pub mod core_macro;
mod av_info;
mod environment;
mod extensions;
mod memory;
mod option_cstr;
mod system_info;

pub use c_utf8;
pub use av_info::*;
pub use environment::*;
pub use extensions::*;
pub use memory::*;
pub use option_cstr::*;
pub use system_info::*;

pub use RetroLoadGameResult::*;

use core::ffi::*;
use core::ops::*;
use sys::*;
use c_utf8::CUtf8;

// u32 is a safe alias for c_uint since libretro.h defines RETRO_ENVIRONMENT_EXPERIMENTAL as 0x10000;
// therefore command values are always larger than a u16, and no platform uses 64-bit c_uint.
pub type EnvironmentCallback = unsafe extern "C" fn(cmd: u32, data: *mut c_void) -> bool;

pub struct NotApplicable();

impl TryFrom<u8> for NotApplicable {
  type Error = ();

  fn try_from(_: u8) -> Result<Self, Self::Error> {
    Err(())
  }
}

#[allow(unused_variables)]
pub trait RetroCore: Sized {
  type SpecialGameType: TryFrom<u8>;
  type SubsystemMemoryType: TryFrom<u8>;

  /// Called during `retro_set_environment`.
  fn set_environment(env: &mut impl SetEnvironmentEnvironment) {}

  /// Called during `retro_init()`. This function is provided for the sake of completeness; it's generally redundant
  /// with [load_game].
  fn init(env: &mut impl InitEnvironment) {}

  /// Called to get information about the core. This information can then be displayed in a frontend, or used to
  /// construct core-specific paths.
  fn get_system_info() -> RetroSystemInfo;

  fn get_system_av_info(&self, env: &mut impl GetSystemAvInfoEnvironment) -> RetroSystemAVInfo;

  fn get_region(&self, env: &mut impl GetRegionEnvironment) -> RetroRegion {
    RetroRegion::NTSC
  }

  /// Called to associate a particular device with a particular port. A core is allowed to ignore this request.
  fn set_controller_port_device(&mut self, env: &mut impl SetPortDeviceEnvironment, port: RetroDevicePort, device: RetroDevice) {}

  /// Called when a player resets their game.
  fn reset(&mut self, env: &mut impl ResetEnvironment);

  /// Called continuously once the core is initialized and a game is loaded. The core is expected to advance emulation
  /// by a single frame before returning.
  fn run(&mut self, env: &mut impl RunEnvironment, runtime: &RetroRuntime);

  /// Called to determine the size of the save state buffer. This is only ever called once per run, and the core must
  /// not exceed the size returned here for subsequent saves.
  fn serialize_size(&self, env: &mut impl SerializeSizeEnvironment) -> usize {
    0
  }

  /// Allows a core to save its internal state into the specified buffer. The buffer is guaranteed to be at least `size`
  /// bytes, where `size` is the value returned from `serialize_size`.
  fn serialize(&self, env: &mut impl SerializeEnvironment, data: &mut [u8]) -> bool {
    false
  }

  /// Allows a core to load its internal state from the specified buffer. The buffer is guaranteed to be at least `size`
  /// bytes, where `size` is the value returned from `serialize_size`.
  fn unserialize(&mut self, env: &mut impl UnserializeEnvironment, data: &[u8]) -> bool {
    false
  }

  fn cheat_reset(&mut self, env: &mut impl CheatResetEnvironment) {}

  fn cheat_set(&mut self, env: &mut impl CheatSetEnvironment, index: u32, enabled: bool, code: &str) {}

  /// Called when a new instance of the core is needed. The `env` parameter can be used to set-up and/or query values
  /// required for the core to function properly.
  fn load_game(env: &mut impl LoadGameEnvironment, game: RetroGame) -> RetroLoadGameResult<Self>;

  fn load_game_special(env: &mut impl LoadGameSpecialEnvironment, game_type: Self::SpecialGameType, info: RetroGame) -> RetroLoadGameResult<Self> {
    Failure
  }

  fn unload_game(&mut self, env: &mut impl UnloadGameEnvironment) {}

  fn get_memory_data(&mut self, env: &mut impl GetMemoryDataEnvironment, id: RetroMemoryType<Self::SubsystemMemoryType>) -> Option<&mut [u8]> {
    None
  }

  fn get_memory_size(&self, env: &mut impl GetMemorySizeEnvironment, id: RetroMemoryType<Self::SubsystemMemoryType>) -> usize {
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

impl TryFrom<c_uint> for RetroDevice {
  type Error = ();

  fn try_from(val: c_uint) -> Result<Self, Self::Error> {
    match val {
      0 => Ok(Self::None),
      1 => Ok(Self::Joypad),
      2 => Ok(Self::Mouse),
      3 => Ok(Self::Keyboard),
      4 => Ok(Self::LightGun),
      5 => Ok(Self::Analog),
      6 => Ok(Self::Pointer),
      _ => Err(()),
    }
  }
}

/// A libretro device port.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RetroDevicePort(u8);

impl RetroDevicePort {
  /// Creates a [RetroDevicePort].
  pub fn new(port_number: u8) -> Self {
    RetroDevicePort(port_number)
  }

  // Converts this port back into a u8.
  pub fn into_inner(self) -> u8 {
    self.0
  }
}

impl From<u8> for RetroDevicePort {
  fn from(port_number: u8) -> Self {
    Self::new(port_number)
  }
}

impl From<RetroDevicePort> for u8 {
  fn from(port: RetroDevicePort) -> Self {
    port.into_inner()
  }
}

/// Represents the possible ways that a frontend can pass game information to a core.
pub enum RetroGame<'a> {
  /// Used if a core supports "no game" and no game was selected.
  ///
  /// * `meta` contains implementation-specific metadata, if present.
  ///
  /// **Note**: "No game" support is hinted with the `RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME` key.
  None { meta: OptionCStr<'a> },
  /// Used if a core doesn't need paths, and a game was selected.
  ///
  /// * `meta` contains implementation-specific metadata, if present.
  /// * `data` contains the entire contents of the game.
  Data { meta: OptionCStr<'a>, data: &'a [u8] },
  /// Used if a core needs paths, and a game was selected.
  ///
  /// * `meta` contains implementation-specific metadata, if present.
  /// * `path` contains the entire absolute path of the game.
  Path { meta: OptionCStr<'a>, path: &'a CUtf8 },
}

impl<'a> From<Option<&retro_game_info>> for RetroGame<'a> {
  fn from(info: Option<&retro_game_info>) -> Self {
    match info {
      None => RetroGame::None { meta: OptionCStr(None) },
      Some(info) => RetroGame::from(info)
    }
  }
}

impl<'a> From<&retro_game_info> for RetroGame<'a> {
  fn from(game: &retro_game_info) -> RetroGame<'a> {
    let meta = if game.meta.is_null() {
      OptionCStr(None)
    } else {
      unsafe { OptionCStr(Some(CStr::from_ptr(game.meta))) }
    };

    match (game.path.is_null(), game.data.is_null()) {
      (true, true) => RetroGame::None { meta },
      (_, false) => unsafe {
        let data = core::slice::from_raw_parts(game.data as *const u8, game.size);
        return RetroGame::Data { meta, data };
      },
      (false, _) => unsafe {
        let path = CUtf8::from_c_str_unchecked(CStr::from_ptr(game.path));
        return RetroGame::Path { meta, path };
      },
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
  L1 = 10,
  R1 = 11,
  L2 = 12,
  R2 = 13,
  L3 = 14,
  R3 = 15,
}

impl From<RetroJoypadButton> for c_uint {
  fn from(button: RetroJoypadButton) -> Self {
    match button {
      RetroJoypadButton::B => 0,
      RetroJoypadButton::Y => 1,
      RetroJoypadButton::Select => 2,
      RetroJoypadButton::Start => 3,
      RetroJoypadButton::Up => 4,
      RetroJoypadButton::Down => 5,
      RetroJoypadButton::Left => 6,
      RetroJoypadButton::Right => 7,
      RetroJoypadButton::A => 8,
      RetroJoypadButton::X => 9,
      RetroJoypadButton::L1 => 10,
      RetroJoypadButton::R1 => 11,
      RetroJoypadButton::L2 => 12,
      RetroJoypadButton::R2 => 13,
      RetroJoypadButton::L3 => 14,
      RetroJoypadButton::R3 => 15,
    }
  }
}

#[must_use]
pub enum RetroLoadGameResult<T> {
  Failure,
  Success(T),
}

impl <T> From<RetroLoadGameResult<T>> for Option<T> where T: RetroCore {
  fn from(result: RetroLoadGameResult<T>) -> Self {
    match result {
      Failure => None,
      Success(core) => Some(core)
    }
  }
}

impl <T> From<Option<T>> for RetroLoadGameResult<T> where T: RetroCore {
  fn from(option: Option<T>) -> Self {
    match option {
      None => Failure,
      Some(core) => Success(core)
    }
  }
}

impl <T, E> From<Result<T, E>> for RetroLoadGameResult<T> where T: RetroCore {
  fn from(result: Result<T, E>) -> Self {
    match result {
      Err(_) => Failure,
      Ok(core) => Success(core)
    }
  }
}

impl <T> From<RetroLoadGameResult<T>> for Result<T, ()> where T: RetroCore {
  fn from(result: RetroLoadGameResult<T>) -> Self {
    match result {
      Failure => Err(()),
      Success(core) => Ok(core)
    }
  }
}

/// Represents the set of regions supported by `libretro`.
#[derive(Clone, Copy)]
pub enum RetroRegion {
  /// A 30 frames/second (60 fields/second) video system.
  NTSC = 0,
  /// A 25 frames/second (50 fields/second) video system.
  PAL = 1,
}

impl From<RetroRegion> for c_uint {
  fn from(region: RetroRegion) -> Self {
    match region {
      RetroRegion::NTSC => 0,
      RetroRegion::PAL => 1,
    }
  }
}

#[derive(Clone, Copy, Debug)]
pub enum RetroPixelFormat {
  RGB1555,
  XRGB8888,
  RGB565,
}

impl From<RetroPixelFormat> for u32 {
  fn from(format: RetroPixelFormat) -> u32 {
    match format {
      RetroPixelFormat::RGB1555 => 0,
      RetroPixelFormat::XRGB8888 => 1,
      RetroPixelFormat::RGB565 => 2,
    }
  }
}

pub struct RetroRuntime {
  audio_sample: retro_audio_sample_t,
  audio_sample_batch: retro_audio_sample_batch_t,
  input_state: retro_input_state_t,
  video_refresh: retro_video_refresh_t,
}

impl RetroRuntime {
  pub fn new(
    audio_sample: retro_audio_sample_t,
    audio_sample_batch: retro_audio_sample_batch_t,
    input_state: retro_input_state_t,
    video_refresh: retro_video_refresh_t,
  ) -> RetroRuntime {
    RetroRuntime {
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
  pub fn is_joypad_button_pressed(&self, port: RetroDevicePort, btn: RetroJoypadButton) -> bool {
    let cb = self
      .input_state
      .expect("`is_joypad_button_pressed` called without registering an `input_state` callback");

    let port = c_uint::from(port.into_inner());
    let device = RETRO_DEVICE_JOYPAD;
    let index = 0;
    let id = btn.into();
    unsafe {
      cb(port, device, index, id) != 0
    }
  }
}

/// This is the glue layer between a `RetroCore` implementation, and the `libretro` API.
pub struct RetroInstance<T: RetroCore> {
  pub system: Option<T>,
  pub audio_sample: retro_audio_sample_t,
  pub audio_sample_batch: retro_audio_sample_batch_t,
  pub environment: retro_environment_t,
  pub input_poll: retro_input_poll_t,
  pub input_state: retro_input_state_t,
  pub video_refresh: retro_video_refresh_t,
}

impl<T: RetroCore> RetroInstance<T> {
  /// Invoked by a `libretro` frontend, with the `retro_get_system_info` API call.
  pub fn on_get_system_info(&mut self, info: &mut retro_system_info) {
    *info = T::get_system_info().into();
  }

  /// Invoked by a `libretro` frontend, with the `retro_get_system_av_info` API call.
  pub fn on_get_system_av_info(&self, info: &mut retro_system_av_info) {
    let system = self.system.as_ref()
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
      if let Ok(port) = u8::try_from(port) {
        let mut env = self.environment();
        let port = RetroDevicePort(port);
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

    let runtime = RetroRuntime::new(
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
      self.core_ref(|core| core.serialize(&mut env, data))
    }
  }

  /// Invoked by a `libretro` frontend, with the `retro_unserialize` API call.
  pub fn on_unserialize(&mut self, data: *const (), size: usize) -> bool {
    unsafe {
      let data = core::slice::from_raw_parts(data as *const u8, size);
      let mut env = self.environment();
      self.core_mut(|core| core.unserialize(&mut env, data))
    }
  }

  /// Invoked by a `libretro` frontend, with the `retro_cheat_reset` API call.
  pub fn on_cheat_reset(&mut self) {
    let mut env = self.environment();
    self.core_mut(|core| core.cheat_reset(&mut env))
  }

  /// Invoked by a `libretro` frontend, with the `retro_cheat_set` API call.
  pub fn on_cheat_set(&mut self, index: c_uint, enabled: bool, code: *const c_char) {
    unsafe {
      let index = u32::from(index);
      let code = CStr::from_ptr(code).to_str().expect("`code` contains invalid data");
      let mut env = self.environment();
      self.core_mut(|core| core.cheat_set(&mut env, index, enabled, code))
    }
  }

  /// Invoked by a `libretro` frontend, with the `retro_load_game` API call.
  pub fn on_load_game(&mut self, game: *const retro_game_info) -> bool {
    let mut env = self.environment();
    let game = RetroGame::from(unsafe { game.as_ref() });
    if let Success(core) = T::load_game(&mut env, game) {
      self.system = Some(core);
      return true;
    }
    false
  }

  /// Invoked by a `libretro` frontend, with the `retro_load_game_special` API call.
  pub fn on_load_game_special(&mut self, game_type: c_uint, info: &retro_game_info, _num_info: usize) -> bool {
    let mut env = self.environment();
    let game_type = u8::try_from(game_type)
      .expect("on_load_game_special() received a game_type outside the expected range.");
    if let Ok(game_type) = T::SpecialGameType::try_from(game_type) {
      if let Success(core) = T::load_game_special(&mut env, game_type, info.into()) {
        self.system = Some(core);
        return true;
      }
    }
    false
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
  pub fn on_get_memory_data(&mut self, id: c_uint) -> *mut () {
    let mut env = self.environment();
    self.core_mut(|core| {
      let id = u16::try_from(id)
        .expect("on_get_memory_data received an id outside of the expected range.");
      if let Ok(id) = RetroMemoryType::try_from(id) {
          if let Some(data) = core.get_memory_data(&mut env, id) {
            // TODO: is there a way to maintain lifetimes here?
            return data.as_mut_ptr() as *mut ()
          }
      }
      core::ptr::null_mut()
    })
  }

  /// Invoked by a `libretro` frontend, with the `retro_get_memory_size` API call.
  pub fn on_get_memory_size(&mut self, id: c_uint) -> usize {
    let mut env = self.environment();
    let id = u16::try_from(id)
      .expect("on_get_memory_size() received an id outside of the expected range.");
    if let Ok(id) = RetroMemoryType::try_from(id) {
      self.core_mut(|core| core.get_memory_size(&mut env, id))
    } else {
      0
    }
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
