#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::redundant_static_lifetimes)]

include!(concat!(env!("OUT_DIR"), "/libretro.rs"));

pub use core::ffi::*;

impl From<&CStr> for retro_variable {
  fn from(c_str: &CStr) -> Self {
    Self {
      key: c_str.as_ptr(),
      value: core::ptr::null(),
    }
  }
}

pub type non_null_retro_audio_sample_t = unsafe extern "C" fn(left: i16, right: i16);
pub type non_null_retro_audio_sample_batch_t = unsafe extern "C" fn(data: *const i16, frames: usize) -> usize;
pub type non_null_retro_environment_t = unsafe extern "C" fn(cmd: c_uint, data: *mut c_void) -> bool;
pub type non_null_retro_input_poll_t = unsafe extern "C" fn();
pub type non_null_retro_input_state_t = unsafe extern "C" fn(port: c_uint, device: c_uint, index: c_uint, id: c_uint) -> i16;
pub type non_null_retro_video_refresh_t = unsafe extern "C" fn(data: *const c_void, width: c_uint, height: c_uint, pitch: usize);

pub type non_null_retro_hw_get_current_framebuffer_t = unsafe extern "C" fn() -> usize;
pub type non_null_retro_hw_get_proc_address_t = unsafe extern "C" fn(sym: *const c_char) -> retro_proc_address_t;
pub type non_null_retro_hw_context_reset_t = unsafe extern "C" fn();

pub const RETRO_HW_FRAME_BUFFER_VALID: *const c_void = sptr::invalid(usize::MAX);

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
