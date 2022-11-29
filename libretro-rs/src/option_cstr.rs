use c_utf8::*;
use core::ffi::*;
use core::ops::*;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OptionCStr<'a>(pub Option<&'a CStr>);

impl <'a> OptionCStr<'a> {
  pub fn as_c_str(&self) -> Option<&'a CStr> {
    self.0
  }

  pub fn as_c_utf8(&self) -> Option<&'a CUtf8> {
    self.0.map(|x| CUtf8::from_c_str(x).ok()).flatten()
  }

  pub fn as_str(&self) -> Option<&'a str> {
    self.as_c_utf8().map(|x| x.as_str())
  }
}

impl <'a> Deref for OptionCStr<'a> {
  type Target = Option<&'a CStr>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}