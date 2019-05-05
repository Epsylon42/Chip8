#[derive(Clone, Copy)]
#[repr(u16)]
pub enum Opcode {
    SetReg = 0x6000,
    SetIndex = 0xA000,
    Disp = 0xD000,
}

impl Opcode {
    pub fn cmp(self, code: u16) -> bool {
        code & self as u16 == self as u16
    }

    pub fn get_arg1(self, code: u16) -> u16 {
        match self {
            Opcode::SetIndex => code & 0x0FFF,
            _ => panic!("Expected opcode with one argument"),
        }
    }

    pub fn get_arg2(self, code: u16) -> (u8, u8) {
        match self {
            Opcode::SetReg => (
                ((code & 0x0F00) >> 8) as u8,
                (code & 0x00FF) as u8
            ),
            _ => panic!("Expected opcode with two arguments"),
        }
    }

    pub fn get_arg3(self, code: u16) -> (u8, u8, u8) {
        match self {
            Opcode::Disp => (
                ((code & 0x0F00) >> 8) as u8,
                ((code & 0x00F0) >> 4) as u8,
                (code & 0x000F) as u8,
            ),
            _ => panic!("Expected opcode with three arguments"),
        }
    }
}

#[macro_export]
macro_rules! match_opcodes {
    ($value:expr; $($rest:tt)*) => {
        loop {
            let value = $value;

            match_opcodes!(@branches{value} $($rest)*);

            unimplemented!("Unknown opcode: {:X}", value);
        }
    };

    (@branches{$value:expr} $x:ident = $opcode:expr => $body:expr, $($rest:tt)*) => {
        if $opcode.cmp($value) {
            let $x = $opcode.get_arg1($value);
            break $body;
        }
        match_opcodes!(@branches{$value} $($rest)*)
    };

    (@branches{$value:expr} ($x1:ident, $x2:ident) = $opcode:expr => $body:expr, $($rest:tt)*) => {
        if $opcode.cmp($value) {
            let ($x1, $x2) = $opcode.get_arg2($value);
            break $body;
        }
        match_opcodes!(@branches{$value} $($rest)*)
    };

    (@branches{$value:expr} ($x1:ident, $x2:ident, $x3:ident) = $opcode:expr => $body:expr, $($rest:tt)*) => {
        if $opcode.cmp($value) {
            let ($x1, $x2, $x3) = $opcode.get_arg3($value);
            break $body;
        }
        match_opcodes!(@branches{$value} $($rest)*)
    };

    (@branches{$value:expr} otherwise $x:ident => $body:expr) => {
        let $x = $value;
        #[allow(unreachable_code)]
        break $body;
    };

    (@branches {$value:expr}) => {};
}
