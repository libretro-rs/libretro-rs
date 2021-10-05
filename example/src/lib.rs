use libretro_rs::*;

pub struct Emulator;

impl RetroCore for Emulator {
  fn new(env: &RetroEnvironment) -> Self {
    let system_dir = env.get_system_directory().unwrap_or("~/.config/emulator");

    println!("[libretro_rs] new(). system_dir={}", system_dir);

    Emulator
  }

  fn get_system_info(_: &mut sys::retro_system_info) {
    println!("[libretro_rs] get_system_info()");
  }

  fn get_system_av_info(&self, _: &RetroEnvironment, _: &mut sys::retro_system_av_info) {
    println!("[libretro_rs] get_system_av_info()");
  }

  fn set_controller_port_device(&mut self, _: &RetroEnvironment, port: u32, device: RetroDevice) {
    println!("[libretro_rs] set_controller_port_device({}, {:?})", port, device);
  }

  fn reset(&mut self, _: &RetroEnvironment) {
    println!("[libretro_rs] reset()");
  }

  fn run(&mut self, _: &RetroEnvironment, _: &RetroRuntime) {
    println!("[libretro_rs] run()");
  }

  fn load_game(&mut self, _: &RetroEnvironment, game: RetroGame) -> bool {
    match game {
      RetroGame::None { .. } => {
        println!("[libretro_rs] load_game()");
        false
      }
      RetroGame::Data { .. } => {
        println!("[libretro_rs] load_game(&[...])");
        true
      }
      RetroGame::Path { path, .. } => {
        println!("[libretro_rs] load_game({})", path);
        true
      }
    }
  }
}

libretro_core!(Emulator);
