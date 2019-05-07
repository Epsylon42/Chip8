#[derive(Clone, Copy)]
#[repr(u16)]
pub enum Opcode {
    ClearScreen = 0x00E0, //

    Return = 0x00EE, //
    Jump = 0x1000, //
    Call = 0x2000, //

    SkipIfEq = 0x3000, //
    SkipIfNeq = 0x4000, //
    SkipIfRegEq = 0x5000, //

    SetReg = 0x6000, //
    SAddReg = 0x7000, //

    MovReg = 0x8000, //
    OrReg = 0x8001, //
    AndReg = 0x8002, //
    XorReg = 0x8003, //
    AddReg = 0x8004, //
    SubReg = 0x8005, //
    RShiftReg = 0x8006, //
    RSubReg = 0x8007, //
    LShiftReg = 0x800E, //

    SkipIfRegNeq = 0x9000, //

    SetIndex = 0xA000, //
    JumpPlus = 0xB000, //
    Rand = 0xC000,

    Draw = 0xD000, //

    SkipIfKeyPressed = 0xE09E, //
    SkipIfKeyNotPressed = 0xE0A1, //

    GetDelay = 0xF007, //
    BlockGetKey = 0xF00A,

    SetDelay = 0xF015, //
    SetSound = 0xF018, //

    AddIndex = 0xF01E, //
    GetSprite = 0xF029, //

    BinCoded = 0xF033, //
    RegDump = 0xF055, //
    RegLoad = 0xF065, //
}

impl Opcode {
    pub fn cmp(self, code: u16) -> bool {
        match self {
            Opcode::ClearScreen |
            Opcode::Return => code == self as u16,

            Opcode::Jump |
            Opcode::Call |
            Opcode::SkipIfEq |
            Opcode::SkipIfNeq |
            Opcode::SetReg |
            Opcode::SAddReg |
            Opcode::SetIndex |
            Opcode::JumpPlus |
            Opcode::Rand |
            Opcode::Draw => code & 0xF000 == self as u16,

            Opcode::SkipIfRegEq |
            Opcode::MovReg |
            Opcode::OrReg |
            Opcode::AndReg |
            Opcode::XorReg |
            Opcode::AddReg |
            Opcode::SubReg |
            Opcode::RShiftReg |
            Opcode::RSubReg |
            Opcode::LShiftReg |
            Opcode::SkipIfRegNeq => code & 0xF00F == self as u16,

            Opcode::SkipIfKeyPressed |
            Opcode::SkipIfKeyNotPressed |
            Opcode::GetDelay |
            Opcode::BlockGetKey |
            Opcode::SetDelay |
            Opcode::SetSound |
            Opcode::AddIndex |
            Opcode::GetSprite |
            Opcode::BinCoded |
            Opcode::RegDump |
            Opcode::RegLoad => code & 0xF0FF == self as u16,
        }
    }

    pub fn get_arg1_u16(self, code: u16) -> u16 {
        match self {
            Opcode::Jump |
            Opcode::Call |
            Opcode::SetIndex |
            Opcode::JumpPlus => code & 0x0FFF,
            _ => panic!("Expected opcode with one 12bit argument"),
        }
    }

    pub fn get_arg1_u8(self, code: u16) -> u8 {
        match self {
            Opcode::SkipIfKeyPressed |
            Opcode::SkipIfKeyNotPressed |
            Opcode::GetDelay |
            Opcode::BlockGetKey |
            Opcode::SetDelay |
            Opcode::SetSound |
            Opcode::AddIndex |
            Opcode::GetSprite |
            Opcode::BinCoded |
            Opcode::RegDump |
            Opcode::RegLoad => ((code & 0x0F00) >> 8) as u8,
            _ => panic!("Expected opcode with one 4bit or 8bit argument"),
        }
    }

    pub fn get_arg2(self, code: u16) -> (u8, u8) {
        match self {
            Opcode::SkipIfEq |
            Opcode::SkipIfNeq |
            Opcode::SetReg |
            Opcode::SAddReg |
            Opcode::Rand => (
                ((code & 0x0F00) >> 8) as u8,
                (code & 0x00FF) as u8
            ),

            Opcode::SkipIfRegEq |
            Opcode::MovReg |
            Opcode::OrReg |
            Opcode::AndReg |
            Opcode::XorReg |
            Opcode::AddReg |
            Opcode::SubReg |
            Opcode::RShiftReg |
            Opcode::RSubReg |
            Opcode::LShiftReg |
            Opcode::SkipIfRegNeq => (
                ((code & 0x0F00) >> 8) as u8,
                ((code & 0x00F0) >> 4) as u8
            ),
            _ => panic!("Expected opcode with two arguments"),
        }
    }

    pub fn get_arg3(self, code: u16) -> (u8, u8, u8) {
        match self {
            Opcode::Draw => (
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

    (@branches{$value:expr} noarg $opcode:expr => $body:expr, $($rest:tt)*) => {
        if $opcode.cmp($value) {
            #[allow(unreachable_code)]
            break $body;
        }
        match_opcodes!(@branches{$value} $($rest)*)
    };

    (@branches{$value:expr} $x:ident = $opcode:expr => $body:expr, $($rest:tt)*) => {
        if $opcode.cmp($value) {
            let $x = $opcode.get_arg1_u8($value);
            #[allow(unreachable_code)]
            break $body;
        }
        match_opcodes!(@branches{$value} $($rest)*)
    };

    (@branches{$value:expr} long $x:ident = $opcode:expr => $body:expr, $($rest:tt)*) => {
        if $opcode.cmp($value) {
            let $x = $opcode.get_arg1_u16($value);
            #[allow(unreachable_code)]
            break $body;
        }
        match_opcodes!(@branches{$value} $($rest)*)
    };

    (@branches{$value:expr} ($x1:ident, $x2:ident) = $opcode:expr => $body:expr, $($rest:tt)*) => {
        if $opcode.cmp($value) {
            let ($x1, $x2) = $opcode.get_arg2($value);
            #[allow(unreachable_code)]
            break $body;
        }
        match_opcodes!(@branches{$value} $($rest)*)
    };

    (@branches{$value:expr} ($x1:ident, $x2:ident, $x3:ident) = $opcode:expr => $body:expr, $($rest:tt)*) => {
        if $opcode.cmp($value) {
            let ($x1, $x2, $x3) = $opcode.get_arg3($value);
            #[allow(unreachable_code)]
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
