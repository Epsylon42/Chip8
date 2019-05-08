use glium::glutin as g;

pub type Key = u8;
pub type Pressed = bool;

pub enum MapKeyResult {
    None,
    Exit,
    Event {
        key: u8,
        pressed: bool,
    }
}

pub fn map_key(ev: g::Event) -> MapKeyResult {
    if let g::Event::WindowEvent {
        event: g::WindowEvent::KeyboardInput {
            input: g::KeyboardInput {
                state,
                virtual_keycode: Some(keycode),
                ..
            },
            ..
        },
        ..
    } = ev {
        let pressed = state == g::ElementState::Pressed;
        let key = match keycode {
            g::VirtualKeyCode::Key1 => 0x1,
            g::VirtualKeyCode::Key2 => 0x2,
            g::VirtualKeyCode::Key3 => 0x3,
            g::VirtualKeyCode::Key4 => 0xC,

            g::VirtualKeyCode::Q => 0x4,
            g::VirtualKeyCode::W => 0x5,
            g::VirtualKeyCode::E => 0x6,
            g::VirtualKeyCode::R => 0xD,

            g::VirtualKeyCode::A => 0x7,
            g::VirtualKeyCode::S => 0x8,
            g::VirtualKeyCode::D => 0x9,
            g::VirtualKeyCode::F => 0xE,

            g::VirtualKeyCode::Z => 0xA,
            g::VirtualKeyCode::X => 0x0,
            g::VirtualKeyCode::C => 0xB,
            g::VirtualKeyCode::V => 0xF,

            g::VirtualKeyCode::Escape => return MapKeyResult::Exit,

            _ => return MapKeyResult::None,
        };

        return MapKeyResult::Event {
            key,
            pressed
        };
    } else {
        MapKeyResult::None
    }
}
