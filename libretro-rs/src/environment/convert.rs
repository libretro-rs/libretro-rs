use crate::ffi::*;
use crate::*;
use c_utf8::CUtf8;
use core::ffi::*;

/// Marker trait for types that are valid arguments to the environment callback.
///
/// Any type implementing this trait must be FFI-safe. Structs should be `#[repr(C)]` or a
/// `#[repr(transparent)]` newtype. Numeric enums should have the appropriate primitive
/// representation, which is typically either `#[repr(u32)]` for `const unsigned` arguments or
/// `#[repr(i32)]` for `const enum` arguments.
///
/// Care must still be taken when calling any of the generic unsafe `[RetroEnvironment]` methods to
/// ensure the type used is appropriate for the environment command, as specified in `libretro.h`.
pub trait RetroEnvironmentData {}
impl RetroEnvironmentData for () {}
impl RetroEnvironmentData for bool {}
impl RetroEnvironmentData for Option<&c_char> {}
impl RetroEnvironmentData for retro_hw_render_callback {}
impl RetroEnvironmentData for retro_log_callback {}
impl RetroEnvironmentData for RetroPixelFormat {}
impl RetroEnvironmentData for RetroGameGeometry {}
impl RetroEnvironmentData for retro_variable {}
impl RetroEnvironmentData for u32 {}

/// Unsafe type conversions.
pub trait RetroEnvironmentResult {
  type Source: RetroEnvironmentData;
  unsafe fn unsafe_from(x: Option<Self::Source>) -> Self;
}

impl RetroEnvironmentResult for bool {
  type Source = bool;

  unsafe fn unsafe_from(x: Option<Self::Source>) -> Self {
    x.unwrap_or(false)
  }
}

impl<'a> RetroEnvironmentResult for Option<&'a CStr> {
  type Source = Option<&'a c_char>;
  unsafe fn unsafe_from(option: Option<Self::Source>) -> Self {
    option.flatten().map(|ptr| CStr::from_ptr(ptr))
  }
}

impl<'a> RetroEnvironmentResult for Option<&'a CUtf8> {
  type Source = Option<&'a c_char>;
  unsafe fn unsafe_from(option: Option<Self::Source>) -> Self {
    option.flatten().map(|ptr| CUtf8::from_c_str_unchecked(CStr::from_ptr(ptr)))
  }
}

impl RetroEnvironmentResult for Option<PlatformLogger> {
  type Source = retro_log_callback;

  unsafe fn unsafe_from(x: Option<Self::Source>) -> Self {
    x.and_then(|cb| cb.log).map(PlatformLogger::new)
  }
}

impl<'a> RetroEnvironmentResult for RetroVariable<'a> {
  type Source = retro_variable;

  unsafe fn unsafe_from(x: Option<Self::Source>) -> Self {
    Self(x.map(|var| CStr::from_ptr(var.value)))
  }
}
