use ::core::ffi::*;

/// A list of file extensions encoded in a pipe-delimited static C string,
/// as specified by the libretro API. Use the [extensions!] macro to create
/// values.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Extensions<'a>(&'a CStr);

impl<'a> Extensions<'a> {
  pub fn new<T: AsRef<CStr> + ?Sized>(str: &'a T) -> Self {
    Self(str.as_ref())
  }

  pub fn as_c_str(&self) -> &CStr {
    self.0
  }

  pub fn as_ptr(&self) -> *const c_char {
    self.0.as_ptr()
  }
}

impl AsRef<CStr> for Extensions<'_> {
  fn as_ref(&self) -> &CStr {
    self.as_c_str()
  }
}

impl From<Extensions<'_>> for *const c_char {
  fn from(extensions: Extensions) -> Self {
    extensions.0.as_ptr()
  }
}

/// Converts a list of file extension string literals into an [Extensions] value.
///
/// # Examples
/// ```
/// use libretro_rs::ext;
/// use libretro_rs::retro::*;
/// use libretro_rs::c_utf8::c_utf8;
/// assert_eq!(ext![], Extensions::new(c_utf8!("")));
/// assert_eq!(ext!["rom"], Extensions::new(c_utf8!("rom")));
/// assert_eq!(ext!["n64", "z64"], Extensions::new(c_utf8!(concat!("n64", "|", "z64"))));
/// ```
#[macro_export]
macro_rules! ext {
  () => { Extensions::new(c_utf8!("")) };
  ( $single:expr ) => { Extensions::new(c_utf8!($single)) };
  ( $head:expr , $( $tail:expr ),+ ) => {
    Extensions::new(c_utf8!(concat!($head, $("|", $tail),+)))
  }
}
