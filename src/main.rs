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
        self.window.draw(self.system.screen(), 64, 32)
    }

    fn display_loop(&mut self) -> Result<(), Error> {
        loop {
            self.draw()?;
            std::thread::sleep(std::time::Duration::from_millis(
                (1.0 / 30.0 * 1000.0f32) as u64,
            ))
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        let delta = std::time::Duration::from_millis((1.0 / 30.0 * 1000.0f32) as u64);

        let mut next_update = std::time::Instant::now() + delta;
        loop {
            let res = self.system.tick();
            if let Err(system::SystemError::ZeroInstruction) = res {
                println!("Reached the end of the program. Entering infinite loop");
                self.display_loop()?;
            } else {
                res?;
            }

            if self.system.dec_timers() {
                println!("Beep!");
            }

            let now = std::time::Instant::now();
            if now >= next_update {
                next_update = now + delta;
                self.draw()?;
            }
        }
    }
}

fn main() {
    let mut chip = Chip8::new().unwrap();

    chip.system
        .load_from_file(std::env::args().skip(1).next().unwrap())
        .unwrap();

    chip.run().unwrap();
}
