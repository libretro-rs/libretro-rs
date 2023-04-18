use crate::retro::*;

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ScreenRotation {
  #[default]
  ZeroDegrees = 0,
  NinetyDegrees = 1,
  OneEightyDegrees = 2,
  TwoSeventyDegrees = 3,
}

impl RetroEnvironmentData for ScreenRotation {}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RetroMessage(retro_message);

impl RetroMessage {
  pub fn new<'a>(msg: impl Into<&'a CUtf8>, frames: u32) -> Self {
    Self(retro_message {
      msg: msg.into().as_ptr(),
      frames,
    })
  }

  pub fn msg(&self) -> &CUtf8 {
    unsafe { CUtf8::from_c_str_unchecked(CStr::from_ptr(self.0.msg)) }
  }

  pub fn frames(&self) -> u32 {
    self.0.frames
  }
}

impl RetroEnvironmentData for RetroMessage {}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RetroVariable<'a>(pub Option<&'a CStr>);

impl<'a> Deref for RetroVariable<'a> {
  type Target = Option<&'a CStr>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
