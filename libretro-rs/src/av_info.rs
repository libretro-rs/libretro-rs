use core::ffi::*;
use core::ops::*;
use libretro_rs_sys::*;

/// Rust interface for [`retro_system_av_info`].
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct RetroSystemAVInfo(retro_system_av_info);

impl RetroSystemAVInfo {
  /// Main constructor.
  pub fn new(geometry: RetroGameGeometry, timing: RetroSystemTiming) -> Self {
    Self(retro_system_av_info {
      geometry: geometry.into(),
      timing: timing.into(),
    })
  }

  /// Returns a [`RetroSystemAVInfo`] with the default [`RetroSystemTiming`].
  pub fn default_timings(geometry: RetroGameGeometry) -> Self {
    Self::new(geometry, RetroSystemTiming::default())
  }

  pub fn geometry(&self) -> RetroGameGeometry {
    RetroGameGeometry(self.0.geometry)
  }

  pub fn timing(&self) -> RetroSystemTiming {
    RetroSystemTiming(self.0.timing)
  }

  pub fn into_inner(self) -> retro_system_av_info {
    self.0
  }
}

impl AsRef<retro_system_av_info> for RetroSystemAVInfo {
  fn as_ref(&self) -> &retro_system_av_info {
    &self.0
  }
}

impl AsMut<retro_system_av_info> for RetroSystemAVInfo {
  fn as_mut(&mut self) -> &mut retro_system_av_info {
    &mut self.0
  }
}

impl From<RetroSystemAVInfo> for retro_system_av_info {
  fn from(av_info: RetroSystemAVInfo) -> Self {
    av_info.into_inner()
  }
}

/// Rust interface for [`retro_game_geometry`].
#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct RetroGameGeometry(retro_game_geometry);

impl RetroGameGeometry {
  /// Creates a [`retro_game_geometry`] with fixed width and height and automatically
  /// derived aspect ratio.
  pub fn fixed(width: u16, height: u16) -> Self {
    Self(retro_game_geometry {
      base_width: width.into(),
      base_height: height.into(),
      max_width: width.into(),
      max_height: height.into(),
      aspect_ratio: 0.0,
    })
  }

  /// Creates a [`retro_game_geometry`] with the given base and max width and height,
  /// and automatically derived aspect ratio.
  pub fn variable(width: RangeInclusive<u16>, height: RangeInclusive<u16>) -> Self {
    Self::new(width, height, 0.0)
  }

  /// Main constructor.
  pub fn new(width: RangeInclusive<u16>, height: RangeInclusive<u16>, aspect_ratio: f32) -> Self {
    Self(retro_game_geometry {
      base_width: c_uint::from(*width.start()),
      base_height: c_uint::from(*height.start()),
      max_width: c_uint::from(*width.end()),
      max_height: c_uint::from(*height.end()),
      aspect_ratio,
    })
  }

  pub fn base_width(&self) -> u16 {
    self.0.base_width as u16
  }

  pub fn base_height(&self) -> u16 {
    self.0.base_height as u16
  }

  pub fn max_width(&self) -> u16 {
    self.0.max_width as u16
  }

  pub fn max_height(&self) -> u16 {
    self.0.max_height as u16
  }

  pub fn aspect_ratio(&self) -> f32 {
    self.0.aspect_ratio
  }

  pub fn into_inner(self) -> retro_game_geometry {
    self.0
  }
}

impl AsRef<retro_game_geometry> for RetroGameGeometry {
  fn as_ref(&self) -> &retro_game_geometry {
    &self.0
  }
}

impl AsMut<retro_game_geometry> for RetroGameGeometry {
  fn as_mut(&mut self) -> &mut retro_game_geometry {
    &mut self.0
  }
}

impl From<RetroGameGeometry> for retro_game_geometry {
  fn from(geometry: RetroGameGeometry) -> Self {
    geometry.into_inner()
  }
}

/// Rust interface for [`retro_system_timing`].
#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct RetroSystemTiming(retro_system_timing);

impl RetroSystemTiming {
  /// Main constructor.
  pub fn new(fps: f64, sample_rate: f64) -> Self {
    Self(retro_system_timing { fps, sample_rate })
  }

  pub fn fps(&self) -> f64 {
    self.0.fps
  }

  pub fn sample_rate(&self) -> f64 {
    self.0.sample_rate
  }

  pub fn into_inner(self) -> retro_system_timing {
    self.0
  }
}

impl Default for RetroSystemTiming {
  /// 60.0 FPS and 44.1khz sample rate.
  fn default() -> Self {
    Self(retro_system_timing {
      fps: 60.0,
      sample_rate: 44_100.0,
    })
  }
}

impl AsRef<retro_system_timing> for RetroSystemTiming {
  fn as_ref(&self) -> &retro_system_timing {
    &self.0
  }
}

impl AsMut<retro_system_timing> for RetroSystemTiming {
  fn as_mut(&mut self) -> &mut retro_system_timing {
    &mut self.0
  }
}

impl From<RetroSystemTiming> for retro_system_timing {
  fn from(timing: RetroSystemTiming) -> Self {
    timing.into_inner()
  }
}
