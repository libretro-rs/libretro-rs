use crate::{
  display::Display,
  keyboard::{Key, Keyboard},
  memory::Memory,
  random::Random,
  stack::Stack,
  timer::Timer,
};

#[derive(Clone, Copy)]
pub struct Reg(usize);

impl Reg {
  pub fn up_to(n: usize) -> Vec<Reg> {
    (0..n).map(Reg).collect()
  }

  pub fn into_key(self) -> Key {
    Key::new(self.0)
  }

  pub fn ordinal(&self) -> usize {
    self.0
  }
}

#[derive(Debug)]
pub struct Instr(u16);

impl Instr {
  pub fn nybbles(&self) -> (u8, u8, u8, u8) {
    let op1 = (self.0 >> (4 * 3)) & 15;
    let op2 = (self.0 >> (4 * 2)) & 15;
    let op3 = (self.0 >> (4 * 1)) & 15;
    let op4 = (self.0 >> (4 * 0)) & 15;

    (op1 as u8, op2 as u8, op3 as u8, op4 as u8)
  }

  pub fn addr(&self) -> u16 {
    self.0 & 0xfff
  }

  pub fn arg(&self) -> u8 {
    self.0 as u8
  }

  pub fn f(&self) -> Reg {
    Reg(0xf)
  }

  pub fn x(&self) -> Reg {
    let idx = (self.0 >> 8) & 15;
    Reg(idx as usize)
  }

  pub fn y(&self) -> Reg {
    let idx = (self.0 >> 4) & 15;
    Reg(idx as usize)
  }

  pub fn z(&self) -> Reg {
    Reg(0)
  }
}

pub struct Cpu {
  pc: u16,
  i: u16,
  v: [u8; 16],
  memory: Memory,
  random: Random,
  stack: Stack,
  pub display: Display,
  pub keyboard: Keyboard,
  pub timer: Timer,
}

impl Cpu {
  pub fn new(game: &[u8]) -> Cpu {
    Cpu {
      pc: 0x200,
      i: 0x000,
      v: [0; 16],
      display: Display::new(),
      keyboard: Keyboard::new(),
      memory: Memory::new(game),
      random: Random::new(),
      stack: Stack::new(),
      timer: Timer::new(),
    }
  }

  fn back(&mut self) {
    self.pc = self.pc.wrapping_sub(2)
  }

  fn skip(&mut self) {
    self.pc = self.pc.wrapping_add(2)
  }

  pub fn read_code(&mut self) -> Instr {
    let value = self.memory.read_word(self.pc);
    self.skip();
    Instr(value)
  }

  pub fn get(&self, reg: Reg) -> u8 {
    self.v[reg.0]
  }

  pub fn set(&mut self, reg: Reg, value: u8) {
    self.v[reg.0] = value
  }

  pub fn step(&mut self) {
    let code = self.read_code();

    match code.nybbles() {
      // 00E0 - CLS
      (0x0, 0x0, 0xe, 0x0) => self.display.cls(),

      // 00EE - RET
      (0x0, 0x0, 0xe, 0xe) => self.pc = self.stack.pull(),

      // 0nnn - SYS
      (0x0, _, _, _) => {
        println!("SYS {:03x} instruction executed", code.addr())
      }

      // 1nnn - JP addr
      (0x1, _, _, _) => self.pc = code.addr(),

      // 2nnn - CALL addr
      (0x2, _, _, _) => {
        self.stack.push(self.pc as u16);
        self.pc = code.addr()
      }

      // 3xkk - SE Vx, byte
      (0x3, _, _, _) => {
        if self.get(code.x()) == code.arg() {
          self.skip()
        }
      }

      // 4xkk - SNE Vx, byte
      (0x4, _, _, _) => {
        if self.get(code.x()) != code.arg() {
          self.skip()
        }
      }

      // 5xy0 - SE Vx, Vy
      (0x5, _, _, 0x0) => {
        if self.get(code.x()) == self.get(code.y()) {
          self.skip()
        }
      }

      // 6xkk - LD Vx, byte
      (0x6, _, _, _) => self.set(code.x(), code.arg()),

      // 7xkk - ADD Vx, byte
      (0x7, _, _, _) => {
        let x = self.get(code.x());
        self.set(code.x(), x.wrapping_add(code.arg()))
      }

      // 8xy0 - LD Vx, Vy
      (0x8, _, _, 0x0) => self.set(code.x(), self.get(code.y())),

      // 8xy1 - OR Vx, Vy
      (0x8, _, _, 0x1) => self.set(code.x(), self.get(code.x()) | self.get(code.y())),

      // 8xy2 - AND Vx, Vy
      (0x8, _, _, 0x2) => self.set(code.x(), self.get(code.x()) & self.get(code.y())),

      // 8xy3 - XOR Vx, Vy
      (0x8, _, _, 0x3) => self.set(code.x(), self.get(code.x()) ^ self.get(code.y())),

      // 8xy4 - ADD Vx, Vy
      (0x8, _, _, 0x4) => {
        let (result, overflow) = self.get(code.x()).overflowing_add(self.get(code.y()));
        self.set(code.f(), u8::from(overflow));
        self.set(code.x(), result);
      }

      // 8xy5 - SUB Vx, Vy
      (0x8, _, _, 0x5) => {
        let (result, overflow) = self.get(code.x()).overflowing_sub(self.get(code.y()));
        self.set(code.f(), u8::from(overflow));
        self.set(code.x(), result)
      }

      // 8xy6 - SHR Vx {, Vy}
      (0x8, _, _, 0x6) => {
        self.set(code.f(), self.get(code.x()) & 1);
        self.set(code.x(), self.get(code.x()) >> 1)
      }

      // 8xy7 - SUBN Vx, Vy
      (0x8, _, _, 0x7) => {
        let (result, overflow) = self.get(code.y()).overflowing_sub(self.get(code.x()));
        self.set(code.f(), u8::from(overflow));
        self.set(code.x(), result);
      }

      // 8xyE - SHL Vx {, Vy}
      (0x8, _, _, 0xe) => {
        self.set(code.f(), (self.get(code.x()) >> 7) & 1);
        self.set(code.x(), self.get(code.x()) << 1);
      }

      // 9xy0 - SNE Vx, Vy
      (0x9, _, _, 0x0) => {
        if self.get(code.x()) != self.get(code.y()) {
          self.skip()
        }
      }

      // Annn - LD I, addr
      (0xa, _, _, _) => self.i = code.addr(),

      // Bnnn - JP V0, addr
      (0xb, _, _, _) => self.pc = code.addr().wrapping_add(self.get(code.z()) as u16),

      // Cxkk - RND Vx, byte
      (0xc, _, _, _) => {
        let val = self.random.next(code.arg());
        self.set(code.x(), val)
      }

      // Dxyn - DRW Vx, Vy, nibble
      (0xd, _, _, n) => {
        let sprite_data: Vec<_> = (0..n as u16)
          .map(|n| self.memory.read_byte(self.i.wrapping_add(n)) as usize)
          .enumerate()
          .collect();

        let vx = self.get(code.x()) as usize;
        let vy = self.get(code.y()) as usize;
        let collisions = self.display.drw(vx, vy, &sprite_data);

        self.set(code.f(), u8::from(collisions));
      }

      // Ex9E - SKP Vx
      (0xe, _, 0x9, 0xe) => {
        let key = code.x().into_key();
        if self.keyboard.key_state(key).is_pressed() {
          self.skip()
        }
      }

      // ExA1 - SKNP Vx
      (0xe, _, 0xa, 0x1) => {
        let key = code.x().into_key();
        if !self.keyboard.key_state(key).is_pressed() {
          self.skip()
        }
      }

      // Fx07 - LD Vx, DT
      (0xf, _, 0x0, 0x7) => self.set(code.x(), self.timer.delay() as u8),

      // Fx0A - LD Vx, K
      (0xf, _, 0x0, 0xa) => {
        if let Some(key) = self.keyboard.first_pressed_key() {
          self.set(code.x(), key.ordinal())
        } else {
          self.back()
        }
      }

      // Fx15 - LD DT, Vx
      (0xf, _, 0x1, 0x5) => self.timer.set_delay(self.get(code.x()) as i32),

      // Fx18 - LD ST, Vx
      (0xf, _, 0x1, 0x8) => self.timer.set_sound(self.get(code.x()) as i32),

      // Fx1E - ADD I, Vx
      (0xf, _, 0x1, 0xe) => self.i = self.i.wrapping_add(self.get(code.x()) as u16),

      // Fx29 - LD F, Vx
      (0xf, _, 0x2, 0x9) => self.i = Memory::glyph_address(self.get(code.x()) as u16),

      // Fx33 - LD B, Vx
      (0xf, _, 0x3, 0x3) => {
        let x = self.get(code.x());

        self.memory.write_byte(self.i.wrapping_add(0), (x / 100) % 10);
        self.memory.write_byte(self.i.wrapping_add(1), (x / 10) % 10);
        self.memory.write_byte(self.i.wrapping_add(2), (x / 1) % 10);
      }

      // Fx55 - LD [I], Vx
      (0xf, len, 0x5, 0x5) => {
        for reg in Reg::up_to(len as usize) {
          let i = reg.ordinal() as u16;
          self.memory.write_byte(self.i.wrapping_add(i), self.get(reg))
        }
      }

      // Fx65 - LD Vx, [I]
      (0xf, len, 0x6, 0x5) => {
        for reg in Reg::up_to(len as usize) {
          let n = reg.ordinal() as u16;
          self.set(reg, self.memory.read_byte(self.i.wrapping_add(n)))
        }
      }

      _ => panic!("Unrecognized instruction: {:?}", code),
    }
  }

  pub fn step_for(&mut self, n: i32) {
    for _ in 0..n {
      self.step();
    }

    self.timer.update();
  }
}
