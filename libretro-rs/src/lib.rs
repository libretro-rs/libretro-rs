pub extern crate libc;
pub extern crate libretro_sys;

// sys::VideoRefreshFn;

#[macro_export]
macro_rules! libretro_core {
  ($core:ty) => {
    use libretro_rs::libc::{c_char, c_uint, size_t};
    use libretro_rs::libretro_sys::*;

    #[no_mangle]
    pub extern fn retro_api_version() -> c_uint {
      API_VERSION
    }

    #[no_mangle]
    pub extern fn retro_get_system_info(info: &mut SystemInfo) {
    }

    #[no_mangle]
    pub extern fn retro_get_system_av_info(info: &mut SystemAvInfo) {
    }

    #[no_mangle]
    pub extern fn retro_init() {
    }

    #[no_mangle]
    pub extern fn retro_deinit() {
    }

    #[no_mangle]
    pub extern fn retro_set_environment(cb: EnvironmentFn) {
    }

    #[no_mangle]
    pub extern fn retro_set_audio_sample(cb: AudioSampleFn) {
    }

    #[no_mangle]
    pub extern fn retro_set_audio_sample_batch(cb: AudioSampleBatchFn) {
    }

    #[no_mangle]
    pub extern fn retro_set_input_poll(cb: InputPollFn) {
    }

    #[no_mangle]
    pub extern fn retro_set_input_state(cb: InputStateFn) {
    }

    #[no_mangle]
    pub extern fn retro_set_video_refresh(cb: VideoRefreshFn) {
    }

    #[no_mangle]
    pub extern fn retro_set_controller_port_device(port: c_uint, device: c_uint) {
    }

    #[no_mangle]
    pub extern fn retro_reset() {
    }

    #[no_mangle]
    pub extern fn retro_run() {
    }

    #[no_mangle]
    pub extern fn retro_serialize_size() -> size_t {
      0
    }

    #[no_mangle]
    pub extern fn retro_serialize(data: *mut (), size: size_t) -> bool {
      false
    }

    #[no_mangle]
    pub extern fn retro_unserialize(data: *const (), size: size_t) -> bool {
      false
    }

    #[no_mangle]
    pub extern fn retro_cheat_reset() {
    }

    #[no_mangle]
    pub extern fn retro_cheat_set(index: c_uint, enabled: bool, code: *const c_char) {
    }

    #[no_mangle]
    pub extern fn retro_load_game(game: &GameInfo) {
    }

    #[no_mangle]
    pub extern fn retro_load_game_special(game_type: c_uint, info: &GameInfo, num_info: size_t) {
    }

    #[no_mangle]
    pub extern fn retro_unload_game() {
    }

    #[no_mangle]
    pub extern fn retro_get_region() -> c_uint {
      0
    }

    #[no_mangle]
    pub extern fn retro_get_memory_data(id: c_uint) -> *mut () {
      std::ptr::null_mut()
    }

    #[no_mangle]
    pub extern fn retro_get_memory_size(id: c_uint) -> size_t {
      0
    }
  };
}
