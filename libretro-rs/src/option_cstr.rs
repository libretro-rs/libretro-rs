use c_utf8::CUtf8;
use core::ffi::*;
use core::ops::*;

/// Newtype for optional C strings; provides conversion methods.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OptionCStr<'a>(pub Option<&'a CStr>);

impl <'a> OptionCStr<'a> {
  /// Maps null pointers to [None]; otherwise applies [CStr::from_ptr].
  /// # Examples
  /// ```
  /// use std::ffi::{CStr, CString};
  /// use libretro_rs::OptionCStr;
  ///
  /// unsafe {
  ///     assert_eq!(OptionCStr::from_ptr(std::ptr::null()), OptionCStr(None));
  ///     let ptr = CString::new("hello!").unwrap().as_ptr();
  ///     assert_eq!(OptionCStr::from_ptr(ptr), OptionCStr(Some(CStr::from_ptr(ptr))))
  /// }
  ///
  /// ```
  pub unsafe fn from_ptr(ptr: *const c_char) -> Self {
    Self(if ptr.is_null() { None } else { Some(CStr::from_ptr(ptr)) })
  }

  /// Returns the inner value.
  pub fn as_c_str(&self) -> Option<&'a CStr> {
    self.0
  }

  /// Applies [CUtf8::from_c_str] to the [CStr], if present. Maps UTF-8 errors to [None].
  /// # Examples
  /// ```
  /// use std::ffi::CString;
  /// use c_utf8::CUtf8;
  /// use libretro_rs::OptionCStr;
  ///
  /// assert_eq!(OptionCStr(None).as_c_utf8(), None);
  /// let not_utf8 = CString::new([255]).unwrap();
  /// assert_eq!(OptionCStr(Some(not_utf8.as_ref())).as_c_utf8(), None);
  /// let utf8 = CString::new("hello!").unwrap();
  /// assert_eq!(
  ///     OptionCStr(Some(utf8.as_ref())).as_c_utf8(),
  ///     Some(CUtf8::from_c_str(utf8.as_ref()).unwrap()));
  /// ```
  pub fn as_c_utf8(&self) -> Option<&'a CUtf8> {
    self.0.map(|x| CUtf8::from_c_str(x).ok()).flatten()
  }

  /// Equivalent to `self.as_c_utf8().map(|x| x.as_str())`.
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