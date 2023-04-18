use crate::ffi::*;
use crate::*;
use c_utf8::CUtf8;
use core::ffi::*;

/// Rust interface for [retro_system_info].
#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct RetroSystemInfo(retro_system_info);

impl RetroSystemInfo {
  /// Minimal constructor. Leaves [RetroSystemInfo::need_fullpath] and
  /// [RetroSystemInfo::block_extract] set to [false].
  pub fn new(library_name: &'static CUtf8, library_version: &'static CUtf8, valid_extensions: Extensions) -> Self {
    Self(retro_system_info {
      library_name: library_name.as_ptr(),
      library_version: library_version.as_ptr(),
      valid_extensions: valid_extensions.as_ptr(),
      need_fullpath: false,
      block_extract: false,
    })
  }

  pub fn with_block_extract(mut self) -> Self {
    self.0.block_extract = true;
    self
  }

  pub fn with_need_full_path(mut self) -> Self {
    self.0.need_fullpath = true;
    self
  }

  pub fn library_name(&self) -> &'static CUtf8 {
    unsafe { Self::ptr_to_str(self.0.library_name) }
  }

  pub fn library_version(&self) -> &'static CUtf8 {
    unsafe { Self::ptr_to_str(self.0.library_version) }
  }

  pub fn valid_extensions(&self) -> Extensions {
    if self.0.valid_extensions.is_null() {
      Extensions(None)
    } else {
      Extensions(Some(unsafe { Self::ptr_to_str(self.0.valid_extensions) }))
    }
  }

  pub fn need_fullpath(&self) -> bool {
    self.0.need_fullpath
  }

  pub fn block_extract(&self) -> bool {
    self.0.block_extract
  }

  pub fn into_inner(self) -> retro_system_info {
    self.0
  }

  unsafe fn ptr_to_str(ptr: *const c_char) -> &'static CUtf8 {
    // Safety: ptr must've come from a &'static CUtf8
    unsafe { CUtf8::from_c_str_unchecked(CStr::from_ptr(ptr)) }
  }
}

impl From<RetroSystemInfo> for retro_system_info {
  fn from(info: RetroSystemInfo) -> Self {
    info.into_inner()
  }
}
