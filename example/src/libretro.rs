use crate::*;

use libretro_rs::c_utf8::c_utf8;
use libretro_rs::prelude::*;
use libretro_rs::{extensions, libretro_core};

pub struct LibretroCore {
  cpu: cpu::Cpu,
  audio_buffer: [i16; timer::AUDIO_BUFFER_SIZE * 2],
  frame_buffer: [u8; display::WIDTH * display::HEIGHT * std::mem::size_of::<u32>()],
}

impl LibretroCore {
  pub fn render_audio(&mut self, runtime: &Runtime) {
    self.cpu.timer.wave(|n, val| {
      self.audio_buffer[(n * 2) + 0] = (val * 32767.0).clamp(-32768.0, 32767.0) as i16;
      self.audio_buffer[(n * 2) + 1] = (val * 32767.0).clamp(-32768.0, 32767.0) as i16;
    });

    runtime.upload_audio_frame(&self.audio_buffer);
  }

  pub fn render_video(&mut self, runtime: &Runtime) {
    const PIXEL_SIZE: usize = 4;
    const PITCH: usize = PIXEL_SIZE * display::WIDTH;

    for y in 0..display::HEIGHT {
      for x in 0..display::WIDTH {
        let color = self.cpu.display.pixel(x, y).into();
        let index = (y * PITCH) + (x * PIXEL_SIZE);

        self.set_rgb(index, color);
      }
    }

    let width = display::WIDTH as u32;
    let height = display::HEIGHT as u32;
    runtime.upload_video_frame(&self.frame_buffer, width, height, PITCH);
  }

  fn set_rgb(&mut self, index: usize, color: Color) {
    self.frame_buffer[index + 0] = color.b;
    self.frame_buffer[index + 1] = color.g;
    self.frame_buffer[index + 2] = color.r;
    self.frame_buffer[index + 3] = 0xff;
  }

  pub fn update_input(&mut self, runtime: &Runtime) {
    for key in keyboard::Keyboard::keys() {
      // todo: chip-8 has a very clunky mapping to a controller.

      let port = DevicePort::new(0);
      let btn = key_to_retro_button(key);
      if runtime.is_joypad_button_pressed(port, btn) {
        self.cpu.keyboard.set_key_state(key, keyboard::KeyState::Pressed)
      } else {
        self.cpu.keyboard.set_key_state(key, keyboard::KeyState::Released)
      }
    }
  }
}

fn key_to_retro_button(key: keyboard::Key) -> JoypadButton {
  match key.ordinal() {
    _ => JoypadButton::Up,
  }
}

impl Core for LibretroCore {
  fn get_system_info() -> SystemInfo {
    SystemInfo::new(c_utf8!("chip8.rs"), c_utf8!(env!("CARGO_PKG_VERSION")), extensions!["png"])
  }

  fn load_game(env: &mut impl env::LoadGame, game: Game) -> Result<Self, LoadGameError> {
    env.set_pixel_format(PixelFormat::XRGB8888)?;
    match game {
      Game::Data { data, .. } => {
        let core = LibretroCore {
          cpu: cpu::Cpu::new(&data),
          audio_buffer: [0; timer::AUDIO_BUFFER_SIZE * 2],
          frame_buffer: [0; display::AREA * std::mem::size_of::<i32>()],
        };
        Ok(core)
      }
      _ => Err(LoadGameError::new()),
    }
  }

  fn get_system_av_info(&self, _env: &mut impl env::GetAvInfo) -> SystemAVInfo {
    const WINDOW_SCALE: u16 = 8;
    const WINDOW_WIDTH: u16 = WINDOW_SCALE * display::WIDTH as u16;
    const WINDOW_HEIGHT: u16 = WINDOW_SCALE * display::HEIGHT as u16;
    SystemAVInfo::default_timings(GameGeometry::fixed(WINDOW_WIDTH, WINDOW_HEIGHT))
  }

  fn reset(&mut self, _env: &mut impl env::Reset) {
    todo!()
  }

  fn run(&mut self, _env: &mut impl env::Run, runtime: &Runtime) {
    self.update_input(runtime);

    self.cpu.step_for(25);

    self.render_audio(runtime);
    self.render_video(runtime);
  }
}

libretro_core!(LibretroCore);

#[derive(Clone, Copy)]
pub struct Color {
  r: u8,
  g: u8,
  b: u8,
}

impl Color {
  pub const BLACK: Color = Color {
    r: 0x00,
    g: 0x00,
    b: 0x00,
  };

  pub const WHITE: Color = Color {
    r: 0xff,
    g: 0xff,
    b: 0xff,
  };
}

impl From<display::Pixel> for Color {
  fn from(pixel: display::Pixel) -> Self {
    match pixel {
      display::Pixel::Off => Self::BLACK,
      display::Pixel::On => Self::WHITE,
    }
  }
}
