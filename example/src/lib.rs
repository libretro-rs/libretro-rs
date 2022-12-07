use libretro_rs::*;

pub struct Emulator;

impl RetroCore for Emulator {
  fn get_system_info() -> RetroSystemInfo {
    println!("[libretro_rs] get_system_info()");

    RetroSystemInfo::new("emulator", env!("CARGO_PKG_VERSION"))
  }

  fn set_controller_port_device(
    &mut self,
    _: &mut RetroEnvironment<SetControllerPortDevice>,
    port: RetroDevicePort,
    device: RetroDevice,
  ) {
    println!("[libretro_rs] set_controller_port_device({:?}, {:?})", port, device);
  }

  fn reset(&mut self, _: &mut RetroEnvironment<Reset>) {
    println!("[libretro_rs] reset()");
  }

  fn run(&mut self, _: &mut RetroEnvironment<Run>, _: &RetroRuntime) {
    println!("[libretro_rs] run()");
  }

  fn load_game(env: &mut RetroEnvironment<LoadGame>, game: RetroGame) -> RetroLoadGameResult<Self> {
    let system_dir = env.get_system_directory().unwrap_or("~/.config/emulator");
    println!("[libretro_rs] load_game(). system_dir={}", system_dir);

    match game {
      RetroGame::None { .. } => {
        println!("[libretro_rs] load_game()");
        return RetroLoadGameResult::Failure;
      }
      RetroGame::Data { .. } => {
        println!("[libretro_rs] load_game(&[...])");
      }
      RetroGame::Path { path, .. } => {
        println!("[libretro_rs] load_game({})", path);
      }
    }

    RetroLoadGameResult::Success {
      region: RetroRegion::NTSC,
      audio: RetroAudioInfo::new(44_100.0),
      video: RetroVideoInfo::new(60.0, 256, 240),
      core: Emulator,
    }
  }
}

libretro_core!(Emulator);
