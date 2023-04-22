use c_utf8::CUtf8;
use std::ffi::CStr;

pub trait IntoCUtf8<'a> {
  fn into_c_utf8(self) -> Option<&'a CUtf8>;
}

impl<'a> IntoCUtf8<'a> for Option<&'a CStr> {
  fn into_c_utf8(self) -> Option<&'a CUtf8> {
    self.and_then(|x| CUtf8::from_c_str(x).ok())
  }
}

pub trait IntoStr<'a> {
  fn into_str(self) -> Option<&'a str>;
}

impl<'a> IntoStr<'a> for Option<&'a CStr> {
  fn into_str(self) -> Option<&'a str> {
    self.and_then(|x| x.to_str().ok())
  }
}
