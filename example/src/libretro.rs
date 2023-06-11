use crate::*;

use crate::keyboard::KeyState;
use libretro_rs::c_utf8::c_utf8;
use libretro_rs::retro::env::{Init, UnloadGame};
use libretro_rs::retro::pixel::{Format, XRGB8888};
use libretro_rs::retro::*;
use libretro_rs::{ext, libretro_core};
use std::error::Error;

pub struct LibretroCore {
  cpu: cpu::Cpu,
  audio_buffer: [i16; timer::AUDIO_BUFFER_SIZE * 2],
  frame_buffer: [XRGB8888; display::WIDTH * display::HEIGHT],
  rendering_mode: SoftwareRenderEnabled,
  pixel_format: Format<XRGB8888>,
}

impl LibretroCore {
  pub fn render_audio(&mut self, runtime: &mut impl Callbacks) {
    self.cpu.timer.wave(|n, val| {
      self.audio_buffer[(n * 2) + 0] = (val * 32767.0).clamp(-32768.0, 32767.0) as i16;
      self.audio_buffer[(n * 2) + 1] = (val * 32767.0).clamp(-32768.0, 32767.0) as i16;
    });

    runtime.upload_audio_frame(&self.audio_buffer);
  }

  pub fn render_video(&mut self, callbacks: &mut impl Callbacks) {
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
    let frame = Frame::new(&self.frame_buffer, width, height);
    callbacks.upload_video_frame(&self.rendering_mode, &self.pixel_format, &frame);
  }

  fn set_rgb(&mut self, index: usize, color: XRGB8888) {
    self.frame_buffer[index] = color;
  }

  pub fn update_input(&mut self, runtime: &mut impl Callbacks) -> InputsPolled {
    let inputs_polled = runtime.poll_inputs();
    for key in keyboard::Keyboard::keys() {
      // todo: chip-8 has a very clunky mapping to a controller.

      let port = DevicePort::new(0);
      let btn = key_to_retro_button(key);
      if runtime.is_joypad_button_pressed(port, btn) {
        self.cpu.keyboard.set_key_state(key, KeyState::Pressed)
      } else {
        self.cpu.keyboard.set_key_state(key, KeyState::Released)
      }
    }
    inputs_polled
  }
}

fn key_to_retro_button(key: keyboard::Key) -> JoypadButton {
  match key.ordinal() {
    _ => JoypadButton::Up,
  }
}

impl<'a> Core<'a> for LibretroCore {
  type Init = ();

  fn get_system_info() -> SystemInfo {
    SystemInfo::new(
      c_utf8!("chip8.rs"),
      c_utf8!(env!("CARGO_PKG_VERSION")),
      ext!["png"],
    )
  }

  fn init(_env: &mut impl Init) -> Self::Init {
    ()
  }

  fn load_game<E: env::LoadGame>(
    game: &GameInfo,
    args: LoadGameExtraArgs<'a, '_, E, Self::Init>,
  ) -> Result<Self, CoreError> {
    let LoadGameExtraArgs {
      env,
      pixel_format,
      rendering_mode,
      ..
    } = args;
    let pixel_format = env.set_pixel_format_xrgb8888(pixel_format)?;
    let data: &[u8] = game.as_data().ok_or(CoreError::new())?.data();
    Ok(Self {
      rendering_mode,
      pixel_format,
      cpu: cpu::Cpu::new(data),
      audio_buffer: [0; timer::AUDIO_BUFFER_SIZE * 2],
      frame_buffer: [XRGB8888::DEFAULT; display::AREA],
    })
  }

  fn get_system_av_info(&self, _env: &mut impl env::GetAvInfo) -> SystemAVInfo {
    const WINDOW_SCALE: u16 = 8;
    const WINDOW_WIDTH: u16 = WINDOW_SCALE * display::WIDTH as u16;
    const WINDOW_HEIGHT: u16 = WINDOW_SCALE * display::HEIGHT as u16;
    SystemAVInfo::default_timings(GameGeometry::fixed(WINDOW_WIDTH, WINDOW_HEIGHT))
  }

  fn run(&mut self, _env: &mut impl env::Run, callbacks: &mut impl Callbacks) -> InputsPolled {
    let inputs_polled = self.update_input(callbacks);

    self.cpu.step_for(25);

    self.render_audio(callbacks);
    self.render_video(callbacks);
    inputs_polled
  }

  fn reset(&mut self, _env: &mut impl env::Reset) {
    todo!()
  }

  fn unload_game(self, _env: &mut impl UnloadGame) -> Self::Init {
    ()
  }
}

libretro_core!(crate::libretro::LibretroCore);

impl From<display::Pixel> for XRGB8888 {
  fn from(pixel: display::Pixel) -> Self {
    match pixel {
      display::Pixel::Off => XRGB8888::DEFAULT,
      display::Pixel::On => XRGB8888::new_with_raw_value(0x00FFFFFF),
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InitCoreError;

impl<T: Error> From<T> for InitCoreError {
  fn from(_value: T) -> Self {
    Self
  }
}
