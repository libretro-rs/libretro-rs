#[macro_export]
macro_rules! libretro_core {
  ($core:ty) => {
    #[doc(hidden)]
    mod __libretro_rs_gen {
      use super::*;
      use core::ffi::c_char;
      use core::ffi::*;
      use libretro_rs::ffi::*;
      use libretro_rs::retro::*;

      static mut RETRO_INSTANCE: RetroInstance<$core> = RetroInstance {
        environment: None,
        audio_sample: None,
        audio_sample_batch: None,
        input_poll: None,
        input_state: None,
        video_refresh: None,
        system: None,
      };

      #[no_mangle]
      extern "C" fn retro_api_version() -> c_uint {
        RETRO_API_VERSION
      }

      #[no_mangle]
      extern "C" fn retro_get_system_info(info: &mut retro_system_info) {
        instance_mut(|instance| instance.on_get_system_info(info))
      }

      #[no_mangle]
      extern "C" fn retro_get_system_av_info(info: &mut retro_system_av_info) {
        instance_ref(|instance| instance.on_get_system_av_info(info))
      }

      #[no_mangle]
      extern "C" fn retro_init() {
        instance_ref(|instance| instance.on_init())
      }

      #[no_mangle]
      extern "C" fn retro_deinit() {
        instance_mut(|instance| instance.on_deinit())
      }

      #[no_mangle]
      extern "C" fn retro_set_environment(cb: EnvironmentCallback) {
        instance_mut(|instance| instance.on_set_environment(cb))
      }

      #[no_mangle]
      extern "C" fn retro_set_audio_sample(cb: retro_audio_sample_t) {
        instance_mut(|instance| instance.on_set_audio_sample(cb))
      }

      #[no_mangle]
      extern "C" fn retro_set_audio_sample_batch(cb: retro_audio_sample_batch_t) {
        instance_mut(|instance| instance.on_set_audio_sample_batch(cb))
      }

      #[no_mangle]
      extern "C" fn retro_set_input_poll(cb: retro_input_poll_t) {
        instance_mut(|instance| instance.on_set_input_poll(cb))
      }

      #[no_mangle]
      extern "C" fn retro_set_input_state(cb: retro_input_state_t) {
        instance_mut(|instance| instance.on_set_input_state(cb))
      }

      #[no_mangle]
      extern "C" fn retro_set_video_refresh(cb: retro_video_refresh_t) {
        instance_mut(|instance| instance.on_set_video_refresh(cb))
      }

      #[no_mangle]
      extern "C" fn retro_set_controller_port_device(port: c_uint, device: c_uint) {
        instance_mut(|instance| instance.on_set_controller_port_device(port, device))
      }

      #[no_mangle]
      extern "C" fn retro_reset() {
        instance_mut(|instance| instance.on_reset())
      }

      #[no_mangle]
      extern "C" fn retro_run() {
        instance_mut(|instance| instance.on_run())
      }

      #[no_mangle]
      extern "C" fn retro_serialize_size() -> usize {
        instance_ref(|instance| instance.on_serialize_size())
      }

      #[no_mangle]
      extern "C" fn retro_serialize(data: *mut (), size: usize) -> bool {
        instance_ref(|instance| instance.on_serialize(data, size))
      }

      #[no_mangle]
      extern "C" fn retro_unserialize(data: *const (), size: usize) -> bool {
        instance_mut(|instance| instance.on_unserialize(data, size))
      }

      #[no_mangle]
      extern "C" fn retro_cheat_reset() {
        instance_mut(|instance| instance.on_cheat_reset())
      }

      #[no_mangle]
      unsafe extern "C" fn retro_cheat_set(index: c_uint, enabled: bool, code: *const c_char) {
        instance_mut(|instance| instance.on_cheat_set(index, enabled, code))
      }

      #[no_mangle]
      unsafe extern "C" fn retro_load_game(game: *const retro_game_info) -> bool {
        instance_mut(|instance| instance.on_load_game(game))
      }

      #[no_mangle]
      extern "C" fn retro_load_game_special(game_type: RetroGameType, info: &retro_game_info, num_info: usize) -> bool {
        instance_mut(|instance| instance.on_load_game_special(game_type, info, num_info))
      }

      #[no_mangle]
      extern "C" fn retro_unload_game() {
        instance_mut(|instance| instance.on_unload_game())
      }

      #[no_mangle]
      extern "C" fn retro_get_region() -> c_uint {
        instance_ref(|instance| instance.on_get_region())
      }

      #[no_mangle]
      extern "C" fn retro_get_memory_data(id: RetroMemoryType) -> *mut () {
        instance_mut(|instance| instance.on_get_memory_data(id))
      }

      #[no_mangle]
      extern "C" fn retro_get_memory_size(id: RetroMemoryType) -> usize {
        instance_mut(|instance| instance.on_get_memory_size(id))
      }

      #[inline]
      fn instance_ref<T>(f: impl FnOnce(&RetroInstance<$core>) -> T) -> T {
        unsafe { f(&RETRO_INSTANCE) }
      }

      #[inline]
      fn instance_mut<T>(f: impl FnOnce(&mut RetroInstance<$core>) -> T) -> T {
        unsafe { f(&mut RETRO_INSTANCE) }
      }
    }
  };
}
