use crate::ffi::*;
use core::ffi::*;
use core::ops::*;

/// Represents the set of regions supported by `libretro`.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Region {
  /// A 30 frames/second (60 fields/second) video system.
  #[default]
  NTSC = 0,
  /// A 25 frames/second (50 fields/second) video system.
  PAL = 1,
}

impl From<Region> for c_uint {
  fn from(region: Region) -> Self {
    region as c_uint
  }
}

/// Rust interface for [`retro_system_av_info`].
#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct SystemAVInfo(retro_system_av_info);

impl SystemAVInfo {
  /// Main constructor.
  pub fn new(geometry: GameGeometry, timing: SystemTiming) -> Self {
    Self(retro_system_av_info {
      geometry: geometry.into(),
      timing: timing.into(),
    })
  }

  /// Returns a [`SystemAVInfo`] with the default [`SystemTiming`].
  pub fn default_timings(geometry: GameGeometry) -> Self {
    Self::new(geometry, SystemTiming::default())
  }

  pub fn geometry(&self) -> GameGeometry {
    GameGeometry(self.0.geometry)
  }

  pub fn timing(&self) -> SystemTiming {
    SystemTiming(self.0.timing)
  }

  pub fn into_inner(self) -> retro_system_av_info {
    self.0
  }
}

impl AsRef<retro_system_av_info> for SystemAVInfo {
  fn as_ref(&self) -> &retro_system_av_info {
    &self.0
  }
}

impl From<SystemAVInfo> for retro_system_av_info {
  fn from(av_info: SystemAVInfo) -> Self {
    av_info.into_inner()
  }
}

/// Rust interface for [`retro_game_geometry`].
#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct GameGeometry(retro_game_geometry);

impl GameGeometry {
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

impl AsRef<retro_game_geometry> for GameGeometry {
  fn as_ref(&self) -> &retro_game_geometry {
    &self.0
  }
}

impl AsMut<retro_game_geometry> for GameGeometry {
  fn as_mut(&mut self) -> &mut retro_game_geometry {
    &mut self.0
  }
}

impl From<GameGeometry> for retro_game_geometry {
  fn from(geometry: GameGeometry) -> Self {
    geometry.into_inner()
  }
}

/// Rust interface for [`retro_system_timing`].
#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct SystemTiming(retro_system_timing);

impl SystemTiming {
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

impl Default for SystemTiming {
  /// 60.0 FPS and 44.1khz sample rate.
  fn default() -> Self {
    Self(retro_system_timing {
      fps: 60.0,
      sample_rate: 44_100.0,
    })
  }
}

impl AsRef<retro_system_timing> for SystemTiming {
  fn as_ref(&self) -> &retro_system_timing {
    &self.0
  }
}

impl AsMut<retro_system_timing> for SystemTiming {
  fn as_mut(&mut self) -> &mut retro_system_timing {
    &mut self.0
  }
}

impl From<SystemTiming> for retro_system_timing {
  fn from(timing: SystemTiming) -> Self {
    timing.into_inner()
  }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PixelFormat {
  #[default]
  RGB1555 = 0,
  XRGB8888 = 1,
  RGB565 = 2,
}

impl From<PixelFormat> for c_int {
  fn from(value: PixelFormat) -> Self {
    value as c_int
  }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ScreenRotation {
  #[default]
  ZeroDegrees = 0,
  NinetyDegrees = 1,
  OneEightyDegrees = 2,
  TwoSeventyDegrees = 3,
}

impl From<ScreenRotation> for c_uint {
  fn from(value: ScreenRotation) -> Self {
    value as c_uint
  }
}

#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct Message(retro_message);

impl Message {
  pub fn new<'a>(msg: impl Into<&'a CStr>, frames: c_uint) -> Self {
    Self(retro_message {
      msg: msg.into().as_ptr(),
      frames,
    })
  }

  pub fn msg(&self) -> &CStr {
    unsafe { CStr::from_ptr(self.0.msg) }
  }

  pub fn frames(&self) -> c_uint {
    self.0.frames
  }
}

impl From<Message> for retro_message {
  fn from(value: Message) -> Self {
    value.0
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GLRenderEnabled(pub(crate) ());

pub trait HWRenderEnabled: private::Sealed {}

impl HWRenderEnabled for GLRenderEnabled {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GLContextCallbacks {
  pub get_proc_address_cb: non_null_retro_hw_get_proc_address_t,
  pub get_current_framebuffer_cb: non_null_retro_hw_get_current_framebuffer_t,
}

mod private {
  use crate::retro::GLRenderEnabled;

  pub trait Sealed {}

  impl Sealed for GLRenderEnabled {}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GLContextType {
  OpenGL2,
  OpenGLCore3_2,
  OpenGLCore3_3,
  OpenGLCore4_0,
  OpenGLCore4_1,
  OpenGLCore4_2,
  OpenGLCore4_3,
  OpenGLCore4_4,
  OpenGLCore4_5,
  OpenGLCore4_6,
  OpenGLES2,
  OpenGLES3,
}

struct GLVersion(retro_hw_context_type, u8, u8);

impl From<GLContextType> for GLVersion {
  fn from(value: GLContextType) -> Self {
    use retro_hw_context_type::*;
    use GLContextType::*;
    match value {
      OpenGL2 => Self(RETRO_HW_CONTEXT_OPENGL, 2, 0),
      OpenGLCore3_2 => Self(RETRO_HW_CONTEXT_OPENGL_CORE, 3, 2),
      OpenGLCore3_3 => Self(RETRO_HW_CONTEXT_OPENGL_CORE, 3, 3),
      OpenGLCore4_0 => Self(RETRO_HW_CONTEXT_OPENGL_CORE, 4, 0),
      OpenGLCore4_1 => Self(RETRO_HW_CONTEXT_OPENGL_CORE, 4, 1),
      OpenGLCore4_2 => Self(RETRO_HW_CONTEXT_OPENGL_CORE, 4, 2),
      OpenGLCore4_3 => Self(RETRO_HW_CONTEXT_OPENGL_CORE, 4, 3),
      OpenGLCore4_4 => Self(RETRO_HW_CONTEXT_OPENGL_CORE, 4, 4),
      OpenGLCore4_5 => Self(RETRO_HW_CONTEXT_OPENGL_CORE, 4, 5),
      OpenGLCore4_6 => Self(RETRO_HW_CONTEXT_OPENGL_CORE, 4, 6),
      OpenGLES2 => Self(RETRO_HW_CONTEXT_OPENGLES2, 2, 0),
      OpenGLES3 => Self(RETRO_HW_CONTEXT_OPENGLES3, 3, 0),
    }
  }
}

#[repr(transparent)]
pub struct GLOptions(retro_hw_render_callback);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GLBufferOptions {
  #[default]
  None,
  DepthOnly,
  DepthAndStencil,
}

impl GLOptions {
  pub fn new(gl_type: GLContextType) -> Self {
    GLOptions(retro_hw_render_callback::default()).set_gl_type(gl_type)
  }

  pub fn set_gl_type(mut self, gl_type: GLContextType) -> Self {
    let GLVersion(context_type, major, minor) = gl_type.into();
    self.0.context_type = context_type;
    self.0.version_major = major as c_uint;
    self.0.version_minor = minor as c_uint;
    self
  }

  pub fn set_bottom_left_origin(mut self, bottom_left_origin: bool) -> Self {
    self.0.bottom_left_origin = bottom_left_origin;
    self
  }

  pub fn set_buffer_options(mut self, buffers: GLBufferOptions) -> Self {
    match buffers {
      GLBufferOptions::None => {
        self.0.depth = false;
        self.0.stencil = false;
      }
      GLBufferOptions::DepthOnly => {
        self.0.depth = true;
        self.0.stencil = false;
      }
      GLBufferOptions::DepthAndStencil => {
        self.0.depth = true;
        self.0.stencil = true;
      }
    }
    self
  }

  pub fn set_cache_context(mut self, cache_context: bool) -> Self {
    self.0.cache_context = cache_context;
    self
  }

  pub fn set_debug_context(mut self, debug_context: bool) -> Self {
    self.0.debug_context = debug_context;
    self
  }
}

impl From<GLOptions> for retro_hw_render_callback {
  fn from(value: GLOptions) -> Self {
    value.0
  }
}
