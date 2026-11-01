use iced_x86::Register;

/// An argument register and the width in bits of its first read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reg {
    // RCX, RDX, R8, R9 or XMM0 to XMM3
    pub register: Register,
    pub bits: u32,
}

impl Reg {
    /// The argument registers of the Windows x64 convention, integer ones first.
    pub const ARGUMENTS: [Register; 8] = [
        Register::RCX,
        Register::RDX,
        Register::R8,
        Register::R9,
        Register::XMM0,
        Register::XMM1,
        Register::XMM2,
        Register::XMM3,
    ];

    pub fn new(register: Register, bits: u32) -> Self {
        Self { register, bits }
    }

    pub fn name(&self) -> &'static str {
        Self::name_of(self.register)
    }

    pub fn from_name(name: &str) -> Option<Register> {
        Self::ARGUMENTS.into_iter().find(|register| Self::name_of(*register) == name)
    }

    pub fn is_xmm(&self) -> bool {
        self.register.is_xmm()
    }

    fn name_of(register: Register) -> &'static str {
        match register {
            Register::RCX => "rcx",
            Register::RDX => "rdx",
            Register::R8 => "r8",
            Register::R9 => "r9",
            Register::XMM0 => "xmm0",
            Register::XMM1 => "xmm1",
            Register::XMM2 => "xmm2",
            Register::XMM3 => "xmm3",
            _ => "?",
        }
    }
}
