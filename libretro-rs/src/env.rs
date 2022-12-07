#![allow(dead_code)]

use std::{ffi::CStr, marker::PhantomData};

use libretro_rs_sys::*;

use crate::{RetroGameGeometry, RetroPixelFormat};

/// Exposes the [`retro_environment_t`] callback in an idiomatic fashion. Each of the `RETRO_ENVIRONMENT_*` keys will
/// eventually have a corresponding method here.
///
/// Until that is accomplished, the keys are available in `libretro_rs::sys` and can be used manually with the `get_raw`,
/// `get`, `get_str`, and `set_raw` functions.
#[derive(Clone, Copy, Debug)]
pub struct RetroEnvironment<State> {
  cb: retro_environment_t,
  phantom: PhantomData<State>,
}

impl<State> RetroEnvironment<State> {
  pub fn from_raw(cb: retro_environment_t) -> Self {
    Self {
      cb,
      phantom: PhantomData,
    }
  }

  /// Used to convert the [`RetroEnvironment`] to a new state.
  pub(crate) fn into_state<NewState>(self) -> RetroEnvironment<NewState> {
    RetroEnvironment {
      cb: self.cb,
      phantom: PhantomData,
    }
  }

  // Complete API here. Methods available in all contexts are `pub`, everything else is not.

  /// Queries a string slice from the environment. A null pointer is interpreted as [`None`].
  pub fn get_str<'a>(&'a self, key: u32) -> Option<&'a str> {
    unsafe {
      let s = self.get(key)?;
      CStr::from_ptr(s).to_str().ok()
    }
  }

  /// Queries a struct from the environment. A null pointer is interpreted as [`None`].
  pub unsafe fn get<T>(&self, key: u32) -> Option<*const T> {
    let mut val: *const T = std::ptr::null();
    if self.get_raw(key, &mut val) && !val.is_null() {
      Some(val)
    } else {
      None
    }
  }

  /// Directly invokes the underlying [`retro_environment_t`] in a "get" configuration.
  #[inline]
  pub unsafe fn get_raw<T>(&self, key: u32, output: *mut *const T) -> bool {
    let cb = self.cb.expect("`get_raw` called without a `retro_environment` callback");
    cb(key, output as *mut libc::c_void)
  }

  /// Directly invokes the underlying [`retro_environment_t`] in a "set" configuration.
  #[inline]
  pub unsafe fn set_raw<T>(&mut self, key: u32, val: *const T) -> bool {
    let cb = self.cb.expect("`set_raw` called without a `retro_environment` callback");
    cb(key, val as *mut libc::c_void)
  }

  /// Directly invokes the underlying [`retro_environment_t`] in a "command" configuration.
  #[inline]
  pub unsafe fn cmd_raw(&mut self, key: u32) -> bool {
    let cb = self.cb.expect("`cmd_raw` called without a `retro_environment` callback");
    cb(key, std::ptr::null_mut())
  }

  /* Environment methods. */

  fn set_rotation(&mut self, _: u32) {
    todo!()
  }

  fn get_overscan(&mut self) -> bool {
    todo!()
  }

  fn get_can_dupe(&mut self) -> bool {
    todo!()
  }

  fn set_message(&mut self, _: retro_message) {
    todo!()
  }

  /// Requests that the frontend shut down. The frontend can refuse to do this, and return `false`.
  pub fn shutdown(&mut self) -> bool {
    unsafe { self.cmd_raw(RETRO_ENVIRONMENT_SHUTDOWN) }
  }

  fn set_performance_level(&mut self, _: u32) {
    todo!()
  }

  /// Queries the path of the system directory.
  pub fn get_system_directory(&self) -> Option<&str> {
    self.get_str(RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY)
  }

  pub fn set_pixel_format(&mut self, val: RetroPixelFormat) -> bool {
    let val: u32 = val.into();
    unsafe { self.set_raw(RETRO_ENVIRONMENT_SET_PIXEL_FORMAT, &val) }
  }

  fn set_input_descriptors(&mut self, _: retro_input_descriptor) {
    todo!()
  }

  fn set_keyboard_callback(&mut self, _: retro_keyboard_callback) {
    todo!()
  }

  fn set_disk_control_interface(&mut self, _: retro_disk_control_callback) {
    todo!()
  }

  fn set_hw_render(&mut self, _: retro_hw_render_callback) {
    todo!()
  }

  fn get_variable(&mut self) -> retro_variable {
    todo!()
  }

  fn set_variables(&mut self, _: retro_variable) {
    todo!()
  }

  fn get_variable_update(&mut self) -> bool {
    todo!()
  }

  pub fn set_support_no_game(&mut self, val: bool) -> bool {
    unsafe { self.set_raw(RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME, &val) }
  }

  /// Queries the path where the current libretro core resides.
  pub fn get_libretro_path(&self) -> Option<&str> {
    self.get_str(RETRO_ENVIRONMENT_GET_LIBRETRO_PATH)
  }

  fn set_frame_time_callback(&mut self, _: retro_frame_time_callback) {
    todo!()
  }

  fn set_audio_callback(&mut self, _: retro_audio_callback) {
    todo!()
  }

  fn get_rumble_interface(&mut self) -> retro_rumble_interface {
    todo!()
  }

  fn get_input_device_capabilities(&mut self) -> u64 {
    todo!()
  }

  #[cfg(experimental)]
  fn get_sensor_interface(&mut self) -> retro_sensor_interface {
    todo!()
  }

  #[cfg(experimental)]
  fn get_camera_interface(&mut self) -> retro_camera_callback {
    todo!()
  }

  fn get_log_interface(&mut self) -> retro_log_callback {
    todo!()
  }

  fn get_perf_interface(&mut self) -> retro_perf_callback {
    todo!()
  }

  fn get_location_interface(&mut self) -> retro_location_callback {
    todo!()
  }

  /// Queries the path of the "core assets" directory.
  pub fn get_core_assets_directory(&self) -> Option<&str> {
    self.get_str(RETRO_ENVIRONMENT_GET_CORE_ASSETS_DIRECTORY)
  }

  /// Queries the path of the save directory.
  pub fn get_save_directory(&self) -> Option<&str> {
    self.get_str(RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY)
  }

  fn set_system_av_info(&mut self, _: retro_system_av_info) {
    todo!()
  }

  fn set_proc_address_callback(&mut self, _: retro_get_proc_address_interface) {
    todo!()
  }

  fn set_subsystem_info(&mut self, _: retro_subsystem_info) {
    todo!()
  }

  fn set_controller_info(&mut self, _: retro_controller_info) {
    todo!()
  }

  #[cfg(experimental)]
  fn set_memory_maps(&mut self, _: retro_memory_map) {
    todo!()
  }

  pub fn set_geometry(&mut self, val: RetroGameGeometry) -> bool {
    let val = val.into();
    unsafe { self.set_raw::<retro_game_geometry>(RETRO_ENVIRONMENT_SET_GEOMETRY, &val) }
  }

  /// Queries the username associated with the frontend.
  pub fn get_username(&self) -> Option<&str> {
    self.get_str(RETRO_ENVIRONMENT_GET_USERNAME)
  }

  fn get_language(&mut self) -> u32 {
    todo!()
  }

  #[cfg(experimental)]
  fn get_current_software_framebuffer(&mut self) -> retro_framebuffer {
    todo!()
  }

  #[cfg(experimental)]
  fn get_hw_render_interface(&mut self) -> Option<retro_hw_render_interface> {
    todo!()
  }

  #[cfg(experimental)]
  fn set_support_achievements(&mut self, _: bool) {
    todo!()
  }

  #[cfg(experimental)]
  fn set_hw_render_context_negotiation_interface(&mut self, _: &retro_hw_render_context_negotiation_interface) {
    todo!()
  }

  fn set_serialization_quirks(&mut self, _: u64) {
    todo!()
  }

  #[cfg(experimental)]
  fn set_hw_shared_context(&mut self) {
    todo!()
  }

  #[cfg(experimental)]
  fn get_vfs_interface(&mut self) -> retro_vfs_interface_info {
    todo!()
  }

  #[cfg(experimental)]
  fn get_led_interface(&mut self) -> retro_led_interface {
    todo!()
  }

  #[cfg(experimental)]
  fn get_audio_video_enable(&mut self) -> i32 {
    todo!()
  }

  #[cfg(experimental)]
  fn get_midi_interface(&mut self) -> Option<retro_midi_interface> {
    todo!()
  }

  #[cfg(experimental)]
  fn get_fastforwarding(&mut self) -> bool {
    todo!()
  }

  #[cfg(experimental)]
  fn get_target_refresh_rate(&mut self) -> f32 {
    todo!()
  }

  #[cfg(experimental)]
  fn get_input_bitmasks(&mut self) -> bool {
    todo!()
  }

  #[cfg(experimental)]
  fn get_core_options_version(&mut self) -> u32 {
    todo!()
  }

  fn set_core_options(&mut self, _: &[retro_core_option_definition]) {
    todo!()
  }

  fn set_core_options_intl(&mut self, _: &retro_core_options_intl) {
    todo!()
  }

  fn set_core_options_display(&mut self, _: &retro_core_option_display) {
    todo!()
  }

  fn get_preferred_hw_render(&mut self) -> u32 {
    todo!()
  }

  fn get_disk_control_interface_version(&mut self) -> u32 {
    todo!()
  }

  fn set_disk_control_ext_interface(&mut self, _: retro_disk_control_ext_callback) {
    todo!()
  }

  fn get_message_interface_version(&mut self, _: u32) {
    todo!()
  }

  fn set_message_ext(&mut self, _: &retro_message_ext) {
    todo!()
  }

  fn get_input_max_users(&mut self) -> u32 {
    todo!()
  }

  fn set_audio_buffer_status_callback(&mut self, _: retro_audio_buffer_status_callback) {
    todo!()
  }

  fn set_minimum_audio_latency(&mut self, _: &u32) {
    todo!()
  }

  fn set_fastforwarding_override(&mut self, _: &retro_fastforwarding_override) {
    todo!()
  }

  fn set_content_info_override(&mut self, _: retro_system_content_info_override) {
    todo!()
  }

  fn get_game_info_ext(&mut self) -> Option<retro_game_info_ext> {
    todo!()
  }

  fn set_core_options_v2(&mut self, _: retro_core_options_v2) {
    todo!()
  }

  fn set_core_options_v2_intl(&mut self, _: retro_core_options_v2_intl) {
    todo!()
  }

  fn set_core_options_update_display_callback(&mut self, _: retro_core_options_update_display_callback) {
    todo!()
  }

  fn set_variable(&mut self, _: retro_variable) {
    todo!()
  }

  #[cfg(experimental)]
  fn get_throttle_state(&mut self) -> retro_throttle_state {
    todo!()
  }
}

// Individual scopes here to expose private methods in the proper context.

#[derive(Clone, Copy, Debug)]
pub enum Global {}

#[derive(Clone, Copy, Debug)]
pub enum Init {}

#[derive(Clone, Copy, Debug)]
pub enum SetControllerPortDevice {}

#[derive(Clone, Copy, Debug)]
pub enum Reset {}

#[derive(Clone, Copy, Debug)]
pub enum Run {}

#[derive(Clone, Copy, Debug)]
pub enum SerializeSize {}

#[derive(Clone, Copy, Debug)]
pub enum Serialize {}

#[derive(Clone, Copy, Debug)]
pub enum Unserialize {}

#[derive(Clone, Copy, Debug)]
pub enum CheatReset {}

#[derive(Clone, Copy, Debug)]
pub enum CheatSet {}

#[derive(Clone, Copy, Debug)]
pub enum LoadGame {}

#[derive(Clone, Copy, Debug)]
pub enum LoadGameSpecial {}

#[derive(Clone, Copy, Debug)]
pub enum UnloadGame {}

#[derive(Clone, Copy, Debug)]
pub enum GetMemoryData {}

#[derive(Clone, Copy, Debug)]
pub enum GetMemorySize {}

// Make a macro for the glue, this becomes an ACL for the different methods.
//
// retro_env!(Global, get_libretro_path, ...);
// retro_env!(LoadGame, get_camera_interface, ...);
