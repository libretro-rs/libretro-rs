use crate::ffi::*;
use ::core::ffi::*;
use c_utf8::CUtf8;

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

impl<'a> Default for Game<'a> {
  fn default() -> Self {
    Game::None { meta: None }
  }
}

impl<'a> From<Option<&retro_game_info>> for Game<'a> {
  fn from(info: Option<&retro_game_info>) -> Self {
    match info {
      None => Game::None { meta: None },
      Some(info) => Game::from(info),
    }
  }
}

impl<'a> From<&retro_game_info> for Game<'a> {
  fn from(game: &retro_game_info) -> Game<'a> {
    let meta = unsafe { game.meta.as_ref().map(|x| CStr::from_ptr(x)) };

    match (game.path.is_null(), game.data.is_null()) {
      (true, true) => Game::None { meta },
      (_, false) => unsafe {
        let data = ::core::slice::from_raw_parts(game.data as *const u8, game.size);
        Game::Data { meta, data }
      },
      (false, _) => unsafe {
        let path = CUtf8::from_c_str_unchecked(CStr::from_ptr(game.path));
        Game::Path { meta, path }
      },
    }
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
