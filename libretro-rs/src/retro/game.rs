use crate::ffi::*;
use crate::retro::convert::*;
use ::core::ffi::*;
use c_utf8::CUtf8;

/// Game data loaded from a file.
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct GameData(retro_game_info);

impl GameData {
  pub unsafe fn from_raw(info: retro_game_info) -> Self {
    Self(info)
  }

  /// The game's data
  pub fn data(&self) -> &[u8] {
    unsafe { core::slice::from_raw_parts(self.0.data.cast(), self.0.size) }
  }

  /// The absolute path to the game file, if available.
  pub fn path(&self) -> Option<&CUtf8> {
    unsafe { self.0.path.as_ref().unsafe_into() }
  }

  /// Implementation-specific metadata.
  pub fn meta(&self) -> Option<&CStr> {
    unsafe { self.0.meta.as_ref().unsafe_into() }
  }
}

/// Full path to a game.
///
/// * `meta` contains implementation-specific metadata, if present.
/// * `path` contains the entire absolute path of the game.
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct GamePath(retro_game_info);

impl GamePath {
  pub unsafe fn from_raw(info: retro_game_info) -> Self {
    Self(info)
  }

  /// The absolute path to the game file, if available.
  pub fn path(&self) -> &CUtf8 {
    unsafe { (&*self.0.path).unsafe_into() }
  }

  /// Implementation-specific metadata.
  pub fn meta(&self) -> Option<&CStr> {
    unsafe { self.0.meta.as_ref().unsafe_into() }
  }
}

/// Game data loaded from a file.
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct SpecialGameData(retro_game_info);

impl SpecialGameData {
  pub unsafe fn from_raw(info: retro_game_info) -> Self {
    Self(info)
  }

  pub fn as_ref(&self) -> Option<&GameData> {
    if self.0.data.is_null() {
      None
    } else {
      Some(unsafe { &*(&self.0 as *const retro_game_info).cast() })
    }
  }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct SpecialGamePath(retro_game_info);

impl SpecialGamePath {
  pub unsafe fn from_raw(info: retro_game_info) -> Self {
    Self(info)
  }

  pub fn as_ref(&self) -> Option<&GamePath> {
    if self.0.path.is_null() {
      None
    } else {
      Some(unsafe { &*(&self.0 as *const retro_game_info).cast() })
    }
  }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GameType(c_uint);

impl GameType {
  pub fn new(n: c_uint) -> Self {
    Self(n)
  }

  pub fn into_inner(self) -> c_uint {
    self.0
  }
}

impl From<c_uint> for GameType {
  fn from(n: c_uint) -> Self {
    Self(n)
  }
}

impl From<GameType> for c_uint {
  fn from(game_type: GameType) -> Self {
    game_type.into_inner()
  }
}
