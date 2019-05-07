use failure::{Error, Fail};

#[macro_use]
mod opcode;
mod fonts;

#[derive(Debug, Fail)]
pub enum SystemError {
    #[fail(display = "Provided program does not fit into memory")]
    ProgramTooLarge,
    #[fail(display = "Tried to access invalid location in memory: {:X}", addr)]
    InvalidMemoryAccess { addr: u16 },
    #[fail(display = "Invalid register access: {:X}", reg)]
    InvalidRegister { reg: u8 },
    #[fail(display = "Stack overflow")]
    StackOverflow,
    #[fail(display = "Stack underflow")]
    StackUnderflow,
    #[fail(display = "Invalid key: {:X}", key)]
    InvalidKey { key: u8 },
    #[fail(display = "Reached zero instruction")]
    ZeroInstruction,
}

const PROGRAM_START: u16 = 0x200;

pub struct Registers {
    pub reg: [u8; 16],
    pub index: u16,
    pub pc: u16,
}

impl Registers {
    pub fn carry(&self) -> u8 {
        self.reg[15]
    }

    pub fn carry_set(&mut self, value: u8) {
        self.reg[15] = value;
    }

    pub fn read(&self, reg: u8) -> Result<u8, SystemError> {
        self.reg
            .get(reg as usize)
            .cloned()
            .ok_or(SystemError::InvalidRegister { reg })
    }

    pub fn write(&mut self, reg: u8, value: u8) -> Result<(), SystemError> {
        if reg > 15 {
            return Err(SystemError::InvalidRegister { reg });
        }

        self.reg[reg as usize] = value;

        Ok(())
    }

    pub fn with<U, F: FnOnce(&mut u8) -> U>(&mut self, reg: u8, func: F) -> Result<U, SystemError> {
        self.reg
            .get_mut(reg as usize)
            .ok_or(SystemError::InvalidRegister { reg })
            .map(func)
    }
}

impl Default for Registers {
    fn default() -> Self {
        Registers {
            reg: [0; 16],
            index: 0,
            pc: PROGRAM_START,
        }
    }
}

#[derive(Default)]
pub struct Timers {
    pub delay: u8,
    pub sound: u8,
}

#[derive(Default)]
pub struct Stack {
    pub stack: [u16; 16],
    pub sp: u16,
}

#[derive(Default)]
pub struct Keys {
    pub keys: [u8; 16],
}

impl Keys {
    pub fn pressed(&self, key: u8) -> Result<bool, SystemError> {
        self.keys
            .get(key as u8 as usize)
            .ok_or(SystemError::InvalidKey { key })
            .map(|key| *key != 0)
    }
}

const SCREEN_WIDTH: u8 = 64;
const SCREEN_HEIGHT: u8 = 32;
const SCREEN_LEN: usize = SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize / 8;

pub struct System {
    pub mem: [u8; 4096],
    pub screen: [u8; SCREEN_LEN],
    pub registers: Registers,
    pub timers: Timers,
    pub stack: Stack,
    pub keys: Keys,
}

impl Default for System {
    fn default() -> Self {
        let mut mem = [0; 4096];
        mem[..fonts::FONTS.len()].copy_from_slice(fonts::FONTS);

        System {
            mem: mem,
            screen: [0; SCREEN_LEN],
            registers: Default::default(),
            timers: Default::default(),
            stack: Default::default(),
            keys: Default::default(),
        }
    }
}

impl System {
    pub fn reset(&mut self) {
        *self = System::default();
    }

    pub fn screen(&self) -> Vec<u8> {
        self.screen
            .iter()
            .flat_map(|x| (0..8).rev().map(move |shift| ((x >> shift) & 1u8) * 255u8))
            .collect()
    }

    pub fn load_from_file(&mut self, path: impl AsRef<std::path::Path>) -> Result<(), Error> {
        self.load(std::fs::File::open(path)?)
    }

    pub fn load(&mut self, mut src: impl std::io::Read) -> Result<(), Error> {
        let mut buf = Vec::new();
        src.read_to_end(&mut buf)?;

        if buf.len() > self.mem.len() - PROGRAM_START as usize {
            bail!(SystemError::ProgramTooLarge);
        }

        self.mem[PROGRAM_START as usize..PROGRAM_START as usize + buf.len()].copy_from_slice(&buf);

        Ok(())
    }

    pub fn tick(&mut self) -> Result<(), SystemError> {
        use opcode::Opcode;

        if self.read_mem_pair(self.registers.pc)? == 0 {
            return Err(SystemError::ZeroInstruction);
        }

        match_opcodes! {
            self.read_mem_pair(self.registers.pc)?;

            noarg Opcode::ClearScreen => {
                self.screen.copy_from_slice(&[0; SCREEN_LEN]);
            },

            noarg Opcode::Return => {
                if self.stack.sp == 0 {
                    return Err(SystemError::StackUnderflow);
                }

                self.stack.sp -= 1;
                self.registers.pc = self.stack.stack[self.stack.sp as usize];
                return Ok(());
            },

            long addr = Opcode::Jump => {
                self.registers.pc = addr;
                return Ok(());
            },

            long addr = Opcode::Call => {
                if self.stack.sp as usize >= self.stack.stack.len() {
                    return Err(SystemError::StackOverflow);
                }

                self.stack.stack[self.stack.sp as usize] = self.registers.pc;
                self.stack.sp += 1;

                self.registers.pc = addr;
                return Ok(());
            },

            (reg, val) = Opcode::SkipIfEq => {
                if self.registers.read(reg)? == val {
                    self.registers.pc += 2;
                }
            },

            (reg, val) = Opcode::SkipIfNeq => {
                if self.registers.read(reg)? != val {
                    self.registers.pc += 2;
                }
            },

            (reg1, reg2) = Opcode::SkipIfRegEq => {
                if self.registers.read(reg1)? == self.registers.read(reg2)? {
                    self.registers.pc += 2;
                }
            },

            (reg1, reg2) = Opcode::SkipIfRegNeq => {
                if self.registers.read(reg1)? != self.registers.read(reg2)? {
                    self.registers.pc += 2;
                }
            },

            (reg, val) = Opcode::SetReg => {
                self.registers.write(reg, val)?;
            },

            (reg, val) = Opcode::SAddReg => {
                let carry = self.registers.with(reg, |reg| {
                    let (new, overflow) = reg.overflowing_add(val);
                    *reg = new;
                    overflow as u8
                })?;
                self.registers.carry_set(carry)
            },

            (reg1, reg2) = Opcode::MovReg => {
                self.registers.write(reg1, self.registers.read(reg2)?)?;
            },

            (reg1, reg2) = Opcode::OrReg => {
                let val = self.registers.read(reg2)?;
                self.registers.with(reg1, |reg| *reg |= val)?;
            },

            (reg1, reg2) = Opcode::AndReg => {
                let val = self.registers.read(reg2)?;
                self.registers.with(reg1, |reg| *reg &= val)?;
            },

            (reg1, reg2) = Opcode::XorReg => {
                let val = self.registers.read(reg2)?;
                self.registers.with(reg1, |reg| *reg ^= val)?;
            },

            (reg1, reg2) = Opcode::AddReg => {
                let val = self.registers.read(reg2)?;
                let carry = self.registers.with(reg1, |reg| {
                    let (new, overflow) = reg.overflowing_add(val);
                    *reg = new;
                    overflow as u8
                })?;
                self.registers.carry_set(carry);
            },

            (reg1, reg2) = Opcode::SubReg => {
                let val = self.registers.read(reg2)?;
                let carry = self.registers.with(reg1, |reg| {
                    let (new, overflow) = reg.overflowing_sub(val);
                    *reg = new;
                    overflow as u8
                })?;
                self.registers.carry_set(carry);
            },

            (reg, _a) = Opcode::RShiftReg => {
                let carry = self.registers.with(reg, |reg| {
                    let bit = *reg & 1;
                    *reg >>= 1;
                    bit
                })?;
                self.registers.carry_set(carry);
            },

            (reg1, reg2) = Opcode::RSubReg => {
                let val = self.registers.read(reg2)?;
                let carry = self.registers.with(reg1, |reg| {
                    let (new, overflow) = val.overflowing_sub(*reg);
                    *reg = new;
                    overflow as u8
                })?;
                self.registers.carry_set(carry);
            },

            (reg, _a) = Opcode::LShiftReg => {
                let carry = self.registers.with(reg, |reg| {
                    let bit = (*reg >> 7) & 1;
                    *reg <<= 1;
                    bit
                })?;
                self.registers.carry_set(carry);
            },

            long x = Opcode::SetIndex => {
                self.registers.index = x;
            },

            long addr = Opcode::JumpPlus => {
                self.registers.pc = self.registers.read(0)? as u16 + addr;
                return Ok(());
            },

            (reg, pattern) = Opcode::Rand => {
                self.registers.write(reg, rand::random::<u8>() & pattern)?;
            },

            reg = Opcode::AddIndex => {
                self.registers.index += self.registers.read(reg)? as u16;
            },

            key = Opcode::SkipIfKeyPressed => {
                if self.keys.pressed(key)? {
                    self.registers.pc += 2;
                }
            },

            key = Opcode::SkipIfKeyNotPressed => {
                if !self.keys.pressed(key)? {
                    self.registers.pc += 2;
                }
            },

            reg = Opcode::GetDelay => {
                self.registers.write(reg, self.timers.delay)?;
            },

            _reg = Opcode::BlockGetKey => {
                unimplemented!("BlockGetKey opcode");
            },

            reg = Opcode::SetDelay => {
                self.timers.delay = self.registers.read(reg)?;
            },

            reg = Opcode::SetSound => {
                self.timers.sound = self.registers.read(reg)?;
            },

            reg = Opcode::GetSprite => {
                self.registers.index = 5 * self.registers.read(reg)? as u16;
            },

            reg = Opcode::BinCoded => {
                let mut val = self.registers.read(reg)?;
                let first = val / 100;
                val %= 100;
                let second = val / 10;
                val %= 10;
                let third = val;

                self.write_mem(self.registers.index, first)?;
                self.write_mem(self.registers.index + 1, second)?;
                self.write_mem(self.registers.index + 2, third)?;
            },

            reg = Opcode::RegDump => {
                for i in 0..=reg {
                    self.write_mem(self.registers.index + i as u16, self.registers.read(i)?)?
                }
            },

            reg = Opcode::RegLoad => {
                for i in 0..=reg {
                    self.registers.write(i, self.read_mem(self.registers.index + i as u16)?)?;
                }
            },

            (x, y, height) = Opcode::Draw => {
                let x = self.registers.read(x)?;
                let y = self.registers.read(y)?;

                let mut carry = false;

                for byte in 0..height {
                    let value: u8 = self.read_mem(self.registers.index + byte as u16)?;
                    for pixel in 0..8 {
                        if self.draw(x + pixel, y + byte, (value >> (7 - pixel)) & 1 != 0) {
                            carry = true;
                        }
                    }
                }

                self.registers.carry_set(carry as u8);
            },

            otherwise x => {
                unimplemented!("Unknown opcode: {:X}", x);
            }
        }

        self.registers.pc += 2;

        Ok(())
    }

    /// decrements delay and sound timers
    /// returns true if sound timer is reduced to zero
    pub fn dec_timers(&mut self) -> bool {
        self.timers.delay = self.timers.delay.saturating_sub(1);
        let prev_sound = self.timers.sound;
        self.timers.sound = self.timers.sound.saturating_sub(1);

        prev_sound != 0 && self.timers.sound == 0
    }

    fn draw(&mut self, x: u8, y: u8, value: bool) -> bool {
        let x_bit = x % 8;
        let x_byte = x / 8;
        if let Some(current_byte) = self
            .screen
            .get_mut(y as usize * SCREEN_WIDTH as usize / 8 + x_byte as usize)
        {
            let current_bit = (*current_byte >> (7 - x_bit)) & 1 != 0;

            *current_byte ^= (value as u8) << (7 - x_bit);

            return current_bit && value;
        }

        return false;
    }

    fn read_mem_pair(&self, ptr: u16) -> Result<u16, SystemError> {
        let fst = *self
            .mem
            .get(ptr as usize)
            .ok_or(SystemError::InvalidMemoryAccess { addr: ptr })?;
        let snd = *self
            .mem
            .get(ptr as usize + 1)
            .ok_or(SystemError::InvalidMemoryAccess { addr: ptr })?;

        Ok((fst as u16) << 8 | snd as u16)
    }

    fn write_mem_pair(&mut self, ptr: u16, data: u16) -> Result<(), SystemError> {
        let fst = (data >> 8) as u8;
        let snd = (data & 0x00FF) as u8;

        if ptr as usize >= self.mem.len() - 1 {
            return Err(SystemError::InvalidMemoryAccess { addr: ptr });
        }

        self.mem[ptr as usize] = fst;
        self.mem[ptr as usize + 1] = snd;

        Ok(())
    }

    fn read_mem(&self, ptr: u16) -> Result<u8, SystemError> {
        self.mem
            .get(ptr as usize)
            .cloned()
            .ok_or(SystemError::InvalidMemoryAccess { addr: ptr })
    }

    fn write_mem(&mut self, ptr: u16, data: u8) -> Result<(), SystemError> {
        if ptr as usize >= self.mem.len() {
            return Err(SystemError::InvalidMemoryAccess { addr: ptr });
        }

        self.mem[ptr as usize] = data;

        Ok(())
    }
}
