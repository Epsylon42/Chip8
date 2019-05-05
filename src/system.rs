use failure::{Error, Fail};

#[macro_use]
mod opcode;

#[derive(Debug, Fail)]
pub enum SystemError {
    #[fail(display = "Provided program does not fit into memory")]
    ProgramTooLarge,
    #[fail(display = "Tried to access invalid location in memory: {:X}", addr)]
    InvalidMemoryAccess { addr: u16 },
    #[fail(display = "Invalid register access: {:X}", reg)]
    InvalidRegister { reg: u8 },
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
        System {
            mem: [0; 4096],
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

        match_opcodes! {
            self.read_mem_pair(self.registers.pc)?;

            (reg, val) = Opcode::SetReg => {
                self.registers.write(reg, val)?;
            },

            x = Opcode::SetIndex => {
                self.registers.index = x;
            },

            (x, y, height) = Opcode::Disp => {
                let x = self.registers.read(x)?;
                let y = self.registers.read(y)?;

                for byte in 0..height {
                    let value: u8 = self.read_mem(self.registers.index + byte as u16)?;
                    for pixel in 0..8 {
                        self.draw(x + pixel, y + byte, (value >> (7 - pixel)) & 1 != 0);
                    }
                }
            },

            otherwise x => {
                unimplemented!("Unknown opcode: {:X}", x);
            }
        }

        self.registers.pc += 2;
        self.timers.delay = self.timers.delay.saturating_sub(1);
        self.timers.sound = self.timers.sound.saturating_sub(1);

        Ok(())
    }

    fn draw(&mut self, x: u8, y: u8, value: bool) -> bool {
        let x_bit = x % 8;
        let x_byte = x / 8;
        if let Some(current_byte) = self
            .screen
            .get_mut(y as usize * SCREEN_WIDTH as usize / 8 + x_byte as usize)
        {
            let current_bit = (*current_byte >> (7 - x_bit)) & 1 != 0;
            let ret = current_bit == true && value == false;

            if value {
                *current_byte |= 1 << (7 - x_bit); // set target bit to 1
            } else {
                *current_byte &= !(1 << (7 - x_bit)); // set target bit to 0
            }

            return ret;
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
