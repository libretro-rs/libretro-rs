use libretro_rs::*;

pub struct Emulator;

impl RetroCore for Emulator {
  fn new(env: &libretro_rs::RetroEnvironment) -> Self {
    println!("[libretro_rs] new()");

    let system_dir = env.get_system_directory().unwrap_or("~/.config/emulator");

    println!("[libretro_rs] system_dir={}", system_dir);

    Emulator
  }

  fn get_system_info(_: &mut libretro_rs::sys::retro_system_info) {
    println!("[libretro_rs] get_system_info()");
  }

  fn get_system_av_info(&self, _: &mut libretro_rs::sys::retro_system_av_info) {
    println!("[libretro_rs] get_system_av_info()");
  }

  fn set_controller_port_device(&mut self, port: u32, device: u32) {
    println!("[libretro_rs] set_controller_port_device({}, {})", port, device);
  }

  fn reset(&mut self) {
    println!("[libretro_rs] reset()");
  }

  fn run(&mut self) {
    println!("[libretro_rs] run()");
  }

  fn load_game(&mut self, game: RetroGame) -> bool {
    match game {
      RetroGame::None => {
        println!("[libretro_rs] load_game()");
        false
      }
      RetroGame::Data(_) => {
        println!("[libretro_rs] load_game(&[...])");
        true
      }
      RetroGame::Path(path) => {
        println!("[libretro_rs] load_game({})", path);
        true
      }
    }
  }
}

libretro_core!(Emulator);
