#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::redundant_static_lifetimes)]

include!(concat!(env!("OUT_DIR"), "/libretro.rs"));

use core::ffi::*;

impl From<&CStr> for retro_variable {
  fn from(c_str: &CStr) -> Self {
    Self {
      key: c_str.as_ptr(),
      value: core::ptr::null(),
    }
  }
}

pub type not_null_retro_environment_t = unsafe extern "C" fn(cmd: c_uint, data: *mut c_void) -> bool;

#[cfg(test)]
mod tests {
  use core::ffi::*;
  use core::mem;

  #[test]
  fn test_c_uint_size() {
    assert!(
      mem::size_of::<c_uint>() >= mem::size_of::<u32>(),
      "libretro requires c_uint to be at least as large as u32"
    )
  }
}
