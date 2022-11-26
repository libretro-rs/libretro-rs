use c_utf8::CUtf8;
use core::ffi::*;
use core::ptr;

/// A list of file extensions encoded in a pipe-delimited static C string,
/// as specified by the libretro API. Use the [extensions!] macro to create
/// values.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Extensions(pub Option<&'static CUtf8>);

impl Extensions {
  pub fn as_ptr(self) -> *const c_char {
    <*const c_char>::from(self)
  }
}

impl From<Extensions> for *const c_char {
  fn from(extensions: Extensions) -> Self {
    extensions.0.map_or(ptr::null(), |str| str.as_ptr())
  }
}

/// Converts a list of file extension string literals into an [Extensions] value.
///
/// # Examples
/// ```
/// use libretro_rs::*;
/// use libretro_rs::c_utf8::c_utf8;
/// assert_eq!(extensions![], Extensions(None));
/// assert_eq!(extensions!["rom"], Extensions(Some(c_utf8!("rom"))));
/// assert_eq!(extensions!["n64", "z64"], Extensions(Some(c_utf8!(concat!("n64", "|", "z64")))));
/// ```
#[macro_export]
macro_rules! extensions {
  () => { Extensions(None) };
  ( $single:expr ) => { Extensions(Some(c_utf8!($single))) };
  ( $head:expr , $( $tail:expr ),+ ) => {
    Extensions(Some(c_utf8!(concat!($head, $("|", $tail),+))))
  }
}