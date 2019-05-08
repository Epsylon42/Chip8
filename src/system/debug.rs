pub struct Debugger {
    enabled: bool
}

impl Debugger {
    pub fn enabled() -> Self {
        Debugger {
            enabled: true,
        }
    }

    pub fn disabled() -> Self {
        Debugger {
            enabled: false,
        }
    }

    pub fn debug(&mut self, s: impl DebugSource) {
        if self.enabled {
            eprintln!("{}", s.get().as_ref());
        }
    }
}

pub trait DebugSource {
    type Out: AsRef<str>;

    fn get(self) -> Self::Out;
}

impl DebugSource for &str {
    type Out = Self;

    fn get(self) -> Self::Out {
        self
    }
}

impl <U: AsRef<str>, T: FnOnce() -> U> DebugSource for T {
    type Out = U;

    fn get(self) -> Self::Out {
        (self)()
    }
}
