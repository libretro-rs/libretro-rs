use libretro_rs::c_utf8::*;
use libretro_rs::sys::*;
use libretro_rs::*;

pub struct Emulator;

impl RetroCore for Emulator {
  type SpecialGameType = NotApplicable;
  type SubsystemMemoryType = NotApplicable;

  fn get_system_info() -> RetroSystemInfo {
    eprintln!("[libretro_rs] get_system_info()");

    RetroSystemInfo::new(c_utf8!("emulator"), c_utf8!(env!("CARGO_PKG_VERSION")), extensions![])
  }

  fn set_controller_port_device(&mut self, _: &mut impl SetPortDeviceEnvironment, port: RetroDevicePort, device: RetroDevice) {
    eprintln!("[libretro_rs] set_controller_port_device({:?}, {:?})", port, device);
  }

  fn reset(&mut self, _: &mut impl ResetEnvironment) {
    eprintln!("[libretro_rs] reset()");
  }

  fn run(&mut self, _: &mut impl RunEnvironment, _: &RetroRuntime) {
    eprintln!("[libretro_rs] run()");
  }

  fn load_game(env: &mut impl LoadGameEnvironment, game: RetroGame) -> RetroLoadGameResult<Self> {
    let system_dir = env.get_system_directory().into_str().unwrap_or("~/.config/emulator");
    eprintln!("[libretro_rs] load_game(). system_dir={}", system_dir);

    match game {
      RetroGame::None { .. } => {
        eprintln!("[libretro_rs] load_game()");
        return Failure;
      }
      RetroGame::Data { .. } => {
        eprintln!("[libretro_rs] load_game(&[...])");
      }
      RetroGame::Path { path, .. } => {
        eprintln!("[libretro_rs] load_game({})", path);
      }
    }

    Success(Emulator)
  }

  fn get_system_av_info(&self, _env: &mut impl GetSystemAvInfoEnvironment) -> RetroSystemAVInfo {
    RetroSystemAVInfo::default_timings(RetroGameGeometry::fixed(256, 240))
  }
}

libretro_core!(Emulator);
