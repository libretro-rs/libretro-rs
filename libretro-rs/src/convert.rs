use crate::ffi::*;
use crate::retro::*;
use ::core::ffi::*;
use ::core::result::Result;
use c_utf8::CUtf8;

/// Unsafe type conversions.
pub trait UnsafeFrom<T> {
  unsafe fn unsafe_from(x: T) -> Self;
}

pub trait UnsafeInto<T> {
  unsafe fn unsafe_into(self) -> T;
}

impl<T, U, E> UnsafeFrom<Result<U, E>> for Result<T, E>
where
  T: UnsafeFrom<U>,
{
  unsafe fn unsafe_from(x: Result<U, E>) -> Self {
    x.map(|ok| T::unsafe_from(ok))
  }
}

impl<T, U> UnsafeInto<U> for T
where
  U: UnsafeFrom<T>,
{
  unsafe fn unsafe_into(self) -> U {
    U::unsafe_from(self)
  }
}

impl<'a> UnsafeFrom<&'a c_char> for &'a CStr {
  unsafe fn unsafe_from(ptr: &'a c_char) -> Self {
    CStr::from_ptr(ptr)
  }
}

impl<'a> UnsafeFrom<&'a c_char> for &'a CUtf8 {
  unsafe fn unsafe_from(ptr: &'a c_char) -> Self {
    CUtf8::from_c_str_unchecked(CStr::from_ptr(ptr))
  }
}

impl<T, R> UnsafeFrom<Option<T>> for Option<R>
where
  R: UnsafeFrom<T>,
{
  unsafe fn unsafe_from(x: Option<T>) -> Self {
    x.map(|x| R::unsafe_from(x))
  }
}

impl UnsafeFrom<retro_log_callback> for PlatformLogger {
  unsafe fn unsafe_from(cb: retro_log_callback) -> Self {
    PlatformLogger::new(cb.log.unwrap())
  }
}
