#[macro_use]
extern crate glium;
#[macro_use]
extern crate failure;

use failure::Error;

pub mod keys;
pub mod system;
pub mod window;

pub struct Chip8 {
    system: system::System,
    window: window::Window,
}

impl Chip8 {
    pub fn new() -> Result<Self, Error> {
        Ok(Chip8 {
            system: system::System::default(),
            window: window::Window::new()?,
        })
    }

    pub fn draw(&mut self) -> Result<(), Error> {
        self.window.draw(
            self.system.screen(),
            64,
            32,
        )
    }
}

fn main() {
    let mut chip = Chip8::new().unwrap();

    chip.system.load(
        std::io::Cursor::new(&[
            0x60, 0x0A,
            0x61, 0x0A,
            0xA2, 0x08,
            0xD0, 0x13,
            0b01011010,
            0b00111100,
            0b01011010,
        ])
    ).unwrap();

    for _ in 0..4 {
        chip.system.tick().unwrap();
    }

    chip.draw().unwrap();

    chip.window.ev.run_forever(|_| glium::glutin::ControlFlow::Continue);
}
