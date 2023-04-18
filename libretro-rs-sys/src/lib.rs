#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::redundant_static_lifetimes)]

include!(concat!(env!("OUT_DIR"), "/libretro.rs"));

impl From<&core::ffi::CStr> for retro_variable {
  fn from(c_str: &core::ffi::CStr) -> Self {
    Self {
      key: c_str.as_ptr(),
      value: core::ptr::null(),
    }
  }
}
