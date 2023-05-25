use crate::convert::*;
use crate::ffi::*;
use crate::option::Option as _;
use c_utf8::CUtf8;
use core::ffi::*;
use core::fmt::{Debug, Formatter};
use core::{ptr, slice};
use std::marker::PhantomData;

/// Game data loaded from a file.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct GameData<'a>(retro_game_info, PhantomData<&'a ()>);

impl<'a> GameData<'a> {
  pub fn new(data: &'a [u8], path: Option<&'a CUtf8>, meta: Option<&'a CStr>) -> Self {
    Self(
      retro_game_info {
        data: data.as_ptr() as *const c_void,
        path: path.map_or_else(ptr::null, &CUtf8::as_ptr),
        meta: meta.map_or_else(ptr::null, &CStr::as_ptr),
        size: data.len(),
      },
      PhantomData,
    )
  }

  pub unsafe fn from_raw(info: retro_game_info) -> Self {
    Self(info, PhantomData)
  }

  /// The game's data
  pub fn data(&self) -> &'a [u8] {
    unsafe { slice::from_raw_parts(self.0.data.cast(), self.0.size) }
  }

  /// The absolute path to the game file, if available.
  pub fn path(&self) -> Option<&'a CUtf8> {
    unsafe { self.0.path.as_ref().unsafe_into() }
  }

  /// Implementation-specific metadata.
  pub fn meta(&self) -> Option<&'a CStr> {
    unsafe { self.0.meta.as_ref().unsafe_into() }
  }
}

impl Debug for GameData<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "GameData({:?})", &self.0)
  }
}

impl AsRef<retro_game_info> for GameData<'_> {
  fn as_ref(&self) -> &retro_game_info {
    &self.0
  }
}

impl From<GameData<'_>> for retro_game_info {
  fn from(game_data: GameData) -> Self {
    game_data.0
  }
}

/// Full path to a game.
///
/// * `meta` contains implementation-specific metadata, if present.
/// * `path` contains the entire absolute path of the game.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct GamePath<'a>(retro_game_info, PhantomData<&'a ()>);

impl<'a> GamePath<'a> {
  pub fn new(path: &'a CUtf8, meta: Option<&'a CStr>) -> Self {
    Self(
      retro_game_info {
        data: ptr::null(),
        path: path.as_ptr(),
        meta: meta.map_or_else(ptr::null, &CStr::as_ptr),
        size: 0,
      },
      PhantomData,
    )
  }

  pub unsafe fn from_raw(info: retro_game_info) -> Self {
    Self(info, PhantomData)
  }

  /// The absolute path to the game file, if available.
  pub fn path(&self) -> &'a CUtf8 {
    unsafe { (&*self.0.path).unsafe_into() }
  }

  /// Implementation-specific metadata.
  pub fn meta(&self) -> Option<&'a CStr> {
    unsafe { self.0.meta.as_ref().unsafe_into() }
  }
}

impl Debug for GamePath<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "GamePath({:?})", &self.0)
  }
}

impl AsRef<retro_game_info> for GamePath<'_> {
  fn as_ref(&self) -> &retro_game_info {
    &self.0
  }
}

impl From<GamePath<'_>> for retro_game_info {
  fn from(value: GamePath) -> Self {
    value.0
  }
}

impl UnsafeFrom<retro_game_info> for GamePath<'_> {
  unsafe fn unsafe_from(x: retro_game_info) -> Self {
    Self::from_raw(x)
  }
}

#[derive(Clone, Copy)]
pub union GameInfo<'a> {
  info: retro_game_info,
  data: GameData<'a>,
  path: GamePath<'a>,
}

#[allow(unused_parens)]
impl<'a> GameInfo<'a> {
  pub fn from_data(data: GameData<'a>) -> Self {
    Self { data }
  }

  pub fn from_path(path: GamePath<'a>) -> Self {
    Self { path }
  }

  pub unsafe fn from_raw(info: retro_game_info) -> Self {
    Self { info }
  }

  pub fn as_ref(&self) -> GameInfoKind<'a, '_> {
    use GameInfoKind::*;
    unsafe { (if self.is_data() { Data(&self.data) } else { Path(&self.path) }) }
  }

  pub fn as_data(&self) -> Option<&GameData<'a>> {
    unsafe { (if self.is_data() { Some(&self.data) } else { None }) }
  }

  pub unsafe fn as_data_unchecked(&self) -> &GameData<'a> {
    &self.data
  }

  pub fn as_path(&self) -> Option<&GamePath<'a>> {
    unsafe { (if self.is_path() { Some(&self.path) } else { None }) }
  }

  pub unsafe fn as_path_unchecked(&self) -> &GamePath<'a> {
    &self.path
  }

  pub fn is_data(&self) -> bool {
    unsafe { !self.info.data.is_null() }
  }

  pub fn is_path(&self) -> bool {
    !self.is_data()
  }
}

impl Debug for GameInfo<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "GameInfo({:?})", unsafe { &self.info })
  }
}

#[derive(Clone, Copy, Debug)]
pub enum GameInfoKind<'a, 'b>
where
  'a: 'b,
{
  Data(&'b GameData<'a>),
  Path(&'b GamePath<'a>),
}

/// Game info for `retro_load_game_special`.
///
/// Unlike [`GameInfo`], a value of this type may not contain any data.
#[derive(Clone, Copy)]
pub union SpecialGameInfo<'a> {
  info: retro_game_info,
  data: GameData<'a>,
  path: GamePath<'a>,
}

#[allow(unused_parens)]
impl<'a> SpecialGameInfo<'a> {
  pub const NONE: Self = Self {
    info: retro_game_info {
      path: ptr::null(),
      data: ptr::null(),
      size: 0,
      meta: ptr::null(),
    },
  };

  pub fn from_data(data: GameData<'a>) -> Self {
    Self { data }
  }

  pub fn from_path(path: GamePath<'a>) -> Self {
    Self { path }
  }

  pub unsafe fn from_raw(info: retro_game_info) -> Self {
    Self { info }
  }

  pub fn as_ref(&self) -> SpecialGameInfoKind<'a, '_> {
    use SpecialGameInfoKind::*;
    unsafe {
      if self.is_data() {
        Data(&self.data)
      } else {
        (if !self.info.path.is_null() { Path(&self.path) } else { None })
      }
    }
  }

  pub fn as_data(&self) -> Option<&GameData<'a>> {
    unsafe { (if self.is_data() { Some(&self.data) } else { None }) }
  }

  pub unsafe fn as_data_unchecked(&self) -> &GameData<'a> {
    &self.data
  }

  pub fn as_path(&self) -> Option<&GamePath<'a>> {
    unsafe { (if self.is_path() { Some(&self.path) } else { None }) }
  }

  pub unsafe fn as_path_unchecked(&self) -> &GamePath<'a> {
    &self.path
  }

  pub fn is_data(&self) -> bool {
    unsafe { !self.info.data.is_null() }
  }

  pub fn is_path(&self) -> bool {
    unsafe { !self.is_data() && !self.info.path.is_null() }
  }

  pub fn is_none(&self) -> bool {
    unsafe { self.info.data.is_null() && self.info.path.is_null() }
  }
}

impl Debug for SpecialGameInfo<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "SpecialGameInfo({:?})", unsafe { &self.info })
  }
}

impl Default for SpecialGameInfo<'_> {
  fn default() -> Self {
    Self::NONE
  }
}

impl<'a> From<GameData<'a>> for SpecialGameInfo<'a> {
  fn from(value: GameData<'a>) -> Self {
    Self::some(value)
  }
}

impl<'a> From<&'a SpecialGameInfo<'a>> for Option<&'a GameData<'a>> {
  fn from(value: &'a SpecialGameInfo) -> Self {
    value.as_data()
  }
}

unsafe impl<'a> crate::option::Option<GameData<'a>> for SpecialGameInfo<'a> {
  const NONE: Self = Self::NONE;

  fn some(data: GameData<'a>) -> Self {
    Self { data }
  }

  fn is_some(&self) -> bool {
    !self.is_none()
  }

  fn is_none(&self) -> bool {
    unsafe { self.info.data.is_null() }
  }

  fn as_ref(&self) -> Option<&GameData<'a>> {
    if self.is_none() {
      None
    } else {
      Some(unsafe { &self.data })
    }
  }

  fn as_mut(&mut self) -> Option<&mut GameData<'a>> {
    if self.is_none() {
      None
    } else {
      Some(unsafe { &mut self.data })
    }
  }

  unsafe fn unwrap_unchecked(self) -> GameData<'a> {
    self.data
  }
}

#[derive(Clone, Copy, Debug)]
pub enum SpecialGameInfoKind<'a, 'b>
where
  'a: 'b,
{
  None,
  Data(&'b GameData<'a>),
  Path(&'b GamePath<'a>),
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
