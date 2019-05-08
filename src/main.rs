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
        let delta = std::time::Duration::from_millis((1.0 / 60.0 * 1000.0f32) as u64);
        let mut debug = system::debug::Debugger::disabled();

        let mut next_update = std::time::Instant::now() + delta;
        loop {
            let res = self.system.tick(&mut debug);
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

            std::thread::sleep(std::time::Duration::from_micros(
                (2400.0f32.recip() * 1000000.0) as u64,
            ));

            let mut err = None;
            let mut exit = false;
            let ev = &mut self.window.ev;
            let sys = &mut self.system;
            ev.poll_events(|event| {
                match keys::map_key(event) {
                    keys::MapKeyResult::Event { key, pressed } => {
                        if let Err(e) = sys.process_key_event(key, pressed) {
                            err = Some(e);
                        }
                    }

                    keys::MapKeyResult::Exit => {
                        exit = true;
                    }

                    keys::MapKeyResult::None => {}
                }
            });
            if exit {
                return Ok(())
            }
            if let Some(err) = err {
                return Err(err.into());
            }
        }
    }

    pub fn run_debug(&mut self) -> Result<(), Error> {
        let mut debug = system::debug::Debugger::enabled();
        loop {
            println!("{}", self.system.registers);
            self.system.tick(&mut debug)?;

            if self.system.dec_timers() {
                println!("Beep!");
            }

            self.draw()?;
            std::io::stdin().read_line(&mut String::new())?;
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
