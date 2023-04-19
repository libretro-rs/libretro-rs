extern crate core;

pub mod av_info;
pub mod convert;
pub mod core_macro;
pub mod environment;
pub mod error;
pub mod extensions;
pub mod logger;
pub mod memory;
pub mod system_info;

use crate::ffi::*;
use crate::prelude::*;
use c_utf8::CUtf8;
use core::ffi::*;
use core::ops::*;

use std::result::Result;

#[allow(unused_variables)]
pub trait Core: Sized {
  /// Called during `retro_set_environment`.
  fn set_environment(env: &mut impl SetEnvironmentEnvironment) {}

  /// Called during `retro_init`. This function is provided for the sake of completeness; it's generally redundant
  /// with [`Core::load_game`].
  fn init(env: &mut impl InitEnvironment) {}

  /// Called to get information about the core. This information can then be displayed in a frontend, or used to
  /// construct core-specific paths.
  fn get_system_info() -> SystemInfo;

  fn get_system_av_info(&self, env: &mut impl GetSystemAvInfoEnvironment) -> SystemAVInfo;

  fn get_region(&self, env: &mut impl GetRegionEnvironment) -> Region {
    Region::NTSC
  }

  /// Called to associate a particular device with a particular port. A core is allowed to ignore this request.
  fn set_controller_port_device(&mut self, env: &mut impl SetPortDeviceEnvironment, port: DevicePort, device: Device) {}

  /// Called when a player resets their game.
  fn reset(&mut self, env: &mut impl ResetEnvironment);

  /// Called continuously once the core is initialized and a game is loaded. The core is expected to advance emulation
  /// by a single frame before returning.
  fn run(&mut self, env: &mut impl RunEnvironment, runtime: &Runtime);

  /// Called to determine the size of the save state buffer. This is only ever called once per run, and the core must
  /// not exceed the size returned here for subsequent saves.
  fn serialize_size(&self, env: &mut impl SerializeSizeEnvironment) -> usize {
    0
  }

  /// Allows a core to save its internal state into the specified buffer. The buffer is guaranteed to be at least `size`
  /// bytes, where `size` is the value returned from `serialize_size`.
  fn serialize(&self, env: &mut impl SerializeEnvironment, data: &mut [u8]) -> Result<(), SerializeError> {
    Err(SerializeError::new())
  }

  /// Allows a core to load its internal state from the specified buffer. The buffer is guaranteed to be at least `size`
  /// bytes, where `size` is the value returned from `serialize_size`.
  fn unserialize(&mut self, env: &mut impl UnserializeEnvironment, data: &[u8]) -> Result<(), UnserializeError> {
    Err(UnserializeError::new())
  }

  fn cheat_reset(&mut self, env: &mut impl CheatResetEnvironment) {}

  fn cheat_set(&mut self, env: &mut impl CheatSetEnvironment, index: u32, enabled: bool, code: &str) {}

  /// Called when a new instance of the core is needed. The `env` parameter can be used to set-up and/or query values
  /// required for the core to function properly.
  fn load_game(env: &mut impl LoadGameEnvironment, game: Game) -> Result<Self, LoadGameError>;

  fn load_game_special(
    env: &mut impl LoadGameSpecialEnvironment,
    game_type: GameType,
    info: Game,
  ) -> Result<Self, LoadGameError> {
    Err(LoadGameError::new())
  }

  fn unload_game(&mut self, env: &mut impl UnloadGameEnvironment) {}

  fn get_memory_data(&mut self, env: &mut impl GetMemoryDataEnvironment, id: MemoryType) -> Option<&mut [u8]> {
    None
  }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GameType(u32);

impl GameType {
  pub fn new(n: u32) -> Self {
    Self(n)
  }

  pub fn into_inner(self) -> u32 {
    self.0
  }
}

impl From<u32> for GameType {
  fn from(n: u32) -> Self {
    Self(n)
  }
}

impl From<GameType> for u32 {
  fn from(game_type: GameType) -> Self {
    game_type.into_inner()
  }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MemoryType(u32);

impl MemoryType {
  pub fn new(n: u32) -> Self {
    Self(n)
  }

  pub fn into_inner(self) -> u32 {
    self.0
  }
}

impl From<u32> for MemoryType {
  fn from(n: u32) -> Self {
    Self(n)
  }
}

impl From<MemoryType> for u32 {
  fn from(memory_type: MemoryType) -> Self {
    memory_type.into_inner()
  }
}

trait TypeId: Sized {
  fn into_discriminant(self) -> u8;
  fn from_discriminant(id: u8) -> Option<Self>;
}

impl TypeId for () {
  fn into_discriminant(self) -> u8 {
    0
  }

  fn from_discriminant(_id: u8) -> Option<Self> {
    None
  }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Device {
  #[default]
  None = 0,
  Joypad = 1,
  Mouse = 2,
  Keyboard = 3,
  LightGun = 4,
  Analog = 5,
  Pointer = 6,
}

impl TryFrom<c_uint> for Device {
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DevicePort(u8);

impl DevicePort {
  /// Creates a [`DevicePort`].
  pub fn new(port_number: u8) -> Self {
    DevicePort(port_number)
  }

  // Converts this port back into a u8.
  pub fn into_inner(self) -> u8 {
    self.0
  }
}

impl From<u8> for DevicePort {
  fn from(port_number: u8) -> Self {
    Self::new(port_number)
  }
}

impl From<DevicePort> for u8 {
  fn from(port: DevicePort) -> Self {
    port.into_inner()
  }
}

/// Represents the possible ways that a frontend can pass game information to a core.
#[derive(Debug, Clone)]
pub enum Game<'a> {
  /// Used if a core supports "no game" and no game was selected.
  ///
  /// * `meta` contains implementation-specific metadata, if present.
  ///
  /// **Note**: "No game" support is hinted with the `RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME` key.
  None { meta: Option<&'a CStr> },
  /// Used if a core doesn't need paths, and a game was selected.
  ///
  /// * `meta` contains implementation-specific metadata, if present.
  /// * `data` contains the entire contents of the game.
  Data { meta: Option<&'a CStr>, data: &'a [u8] },
  /// Used if a core needs paths, and a game was selected.
  ///
  /// * `meta` contains implementation-specific metadata, if present.
  /// * `path` contains the entire absolute path of the game.
  Path { meta: Option<&'a CStr>, path: &'a CUtf8 },
}

impl<'a> From<Option<&retro_game_info>> for Game<'a> {
  fn from(info: Option<&retro_game_info>) -> Self {
    match info {
      None => Game::None { meta: None },
      Some(info) => Game::from(info),
    }
  }
}

impl<'a> Default for Game<'a> {
  fn default() -> Self {
    Game::None { meta: None }
  }
}

impl<'a> From<&retro_game_info> for Game<'a> {
  fn from(game: &retro_game_info) -> Game<'a> {
    let meta = unsafe { game.meta.as_ref().map(|x| CStr::from_ptr(x)) };

    match (game.path.is_null(), game.data.is_null()) {
      (true, true) => Game::None { meta },
      (_, false) => unsafe {
        let data = core::slice::from_raw_parts(game.data as *const u8, game.size);
        Game::Data { meta, data }
      },
      (false, _) => unsafe {
        let path = CUtf8::from_c_str_unchecked(CStr::from_ptr(game.path));
        Game::Path { meta, path }
      },
    }
  }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum JoypadButton {
  #[default]
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
  #[cfg(experimental)]
  Mask = 256,
}

impl From<JoypadButton> for u32 {
  fn from(button: JoypadButton) -> u32 {
    match button {
      JoypadButton::B => 0,
      JoypadButton::Y => 1,
      JoypadButton::Select => 2,
      JoypadButton::Start => 3,
      JoypadButton::Up => 4,
      JoypadButton::Down => 5,
      JoypadButton::Left => 6,
      JoypadButton::Right => 7,
      JoypadButton::A => 8,
      JoypadButton::X => 9,
      JoypadButton::L1 => 10,
      JoypadButton::R1 => 11,
      JoypadButton::L2 => 12,
      JoypadButton::R2 => 13,
      JoypadButton::L3 => 14,
      JoypadButton::R3 => 15,
      #[cfg(experimental)]
      JoypadButton::Mask => 256,
    }
  }
}

/// Represents the set of regions supported by `libretro`.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Region {
  /// A 30 frames/second (60 fields/second) video system.
  #[default]
  NTSC = 0,
  /// A 25 frames/second (50 fields/second) video system.
  PAL = 1,
}

impl From<Region> for c_uint {
  fn from(region: Region) -> Self {
    match region {
      Region::NTSC => 0,
      Region::PAL => 1,
    }
  }
}

#[repr(i32)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PixelFormat {
  #[default]
  RGB1555 = 0,
  XRGB8888 = 1,
  RGB565 = 2,
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
