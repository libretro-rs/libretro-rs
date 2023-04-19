use crate::ffi::*;
use crate::retro::*;
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
pub trait CommandData {}
impl CommandData for () {}
impl CommandData for bool {}
impl CommandData for Option<&c_char> {}
impl CommandData for retro_hw_render_callback {}
impl CommandData for retro_log_callback {}
impl CommandData for PixelFormat {}
impl CommandData for retro_game_geometry {}
impl CommandData for retro_variable {}
impl CommandData for retro_message {}
impl CommandData for u32 {}

/// Unsafe type conversions.
pub trait CommandOutput {
  type Source: CommandData;
  unsafe fn unsafe_from(x: Self::Source) -> Self;
}

impl CommandOutput for bool {
  type Source = bool;

  unsafe fn unsafe_from(x: Self::Source) -> Self {
    x
  }
}

impl<'a> CommandOutput for Option<&'a CStr> {
  type Source = Option<&'a c_char>;

  unsafe fn unsafe_from(option: Self::Source) -> Self {
    option.map(|ptr| CStr::from_ptr(ptr))
  }
}

impl<'a> CommandOutput for Option<&'a CUtf8> {
  type Source = Option<&'a c_char>;

  unsafe fn unsafe_from(option: Self::Source) -> Self {
    option.map(|ptr| CUtf8::from_c_str_unchecked(CStr::from_ptr(ptr)))
  }
}

impl CommandOutput for PlatformLogger {
  type Source = retro_log_callback;

  unsafe fn unsafe_from(x: Self::Source) -> Self {
    PlatformLogger::new(x.log.unwrap())
  }
}

impl<'a> CommandOutput for RetroVariable<'a> {
  type Source = retro_variable;

  unsafe fn unsafe_from(x: Self::Source) -> Self {
    Self(x.value.as_ref().map(|ptr| CStr::from_ptr(ptr)))
  }
}
