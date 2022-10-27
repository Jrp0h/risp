#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Operation {
    Nop = 0,
    Push = 1,
    Pop = 2,
    Mov = 3,
    Jmp = 4,
    Dup = 5,
    Add = 6,
    Sub = 7,
    Mult = 8,
    Div = 9,
    JmpIf = 10,

    CmpEq = 11, // same as cmp
    CmpNe = 12,
    CmpGt = 13,
    CmpLt = 14,
    CmpGte = 15,
    CmpLte = 16,

    Mod = 17,
    Call = 18,
    Ret = 19,
    Not = 20,
}

impl Operation {
    pub fn from_usize(value: usize) -> Option<Operation> {
        match value {
            0 => Some(Operation::Nop),
            1 => Some(Operation::Push),
            2 => Some(Operation::Pop),
            3 => Some(Operation::Mov),
            4 => Some(Operation::Jmp),
            5 => Some(Operation::Dup),
            6 => Some(Operation::Add),
            7 => Some(Operation::Sub),
            8 => Some(Operation::Mult),
            9 => Some(Operation::Div),
            10 => Some(Operation::JmpIf),
            11 => Some(Operation::CmpEq),
            12 => Some(Operation::CmpNe),
            13 => Some(Operation::CmpGt),
            14 => Some(Operation::CmpLt),
            15 => Some(Operation::CmpGte),
            16 => Some(Operation::CmpLte),
            17 => Some(Operation::Mod),
            18 => Some(Operation::Call),
            19 => Some(Operation::Ret),
            20 => Some(Operation::Not),
            _ => None,
        }
    }

    pub fn from_asm(value: &str) -> Option<Operation> {
        match value {
            "nop" => Some(Operation::Nop),
            "push" => Some(Operation::Push),
            "pop" => Some(Operation::Pop),
            "mov" => Some(Operation::Mov),
            "jmp" => Some(Operation::Jmp),
            "dup" => Some(Operation::Dup),
            "add" => Some(Operation::Add),
            "sub" => Some(Operation::Sub),
            "mult" => Some(Operation::Mult),
            "div" => Some(Operation::Div),
            "jmp_if" => Some(Operation::JmpIf),
            "cmp_eq" => Some(Operation::CmpEq),
            "cmp_ne" => Some(Operation::CmpNe),
            "cmp_gt" => Some(Operation::CmpGt),
            "cmp_lt" => Some(Operation::CmpLt),
            "cmp_gte" => Some(Operation::CmpGte),
            "cmp_lte" => Some(Operation::CmpLte),
            "mod" => Some(Operation::Mod),
            "call" => Some(Operation::Call),
            "ret" => Some(Operation::Ret),
            "not" => Some(Operation::Not),
            _ => None,
        }
    }

    pub fn to_asm(&self) -> &'static str {
        match self {
            Operation::Nop => "nop",
            Operation::Push => "push",
            Operation::Pop => "pop",
            Operation::Mov => "mov",
            Operation::Jmp => "jmp",
            Operation::Dup => "dup",
            Operation::Add => "add",
            Operation::Sub => "sub",
            Operation::Mult => "mult",
            Operation::Div => "div",
            Operation::JmpIf => "jmp_if",
            Operation::CmpEq => "cmp_eq",
            Operation::CmpNe => "cmp_ne",
            Operation::CmpGt => "cmp_gt",
            Operation::CmpLt => "cmp_lt",
            Operation::CmpGte => "cmp_gte",
            Operation::CmpLte => "cmp_lte",
            Operation::Mod => "mod",
            Operation::Call => "call",
            Operation::Ret => "ret",
            Operation::Not => "not",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Variant {
    None = 0,
    Register = 1,
    Direct = 2,
    Indirect = 3,
    Stack = 4,
    StackRelative = 5,
    Native = 6,
}

impl Variant {
    pub fn from_usize(value: usize) -> Option<Variant> {
        match value {
            0 => Some(Variant::None),
            1 => Some(Variant::Register),
            2 => Some(Variant::Direct),
            3 => Some(Variant::Indirect),
            4 => Some(Variant::Stack),
            5 => Some(Variant::StackRelative),
            6 => Some(Variant::Native),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct OpCode(usize);

impl OpCode {
    pub fn new(operation: Operation, variants: [Variant; 3]) -> Self {
        // TODO: Make operation 5bits
        let mut code = (operation as usize) << (8 * 4);
        code |= (variants[0] as usize) << (8 * 2);
        code |= (variants[1] as usize) << (8 * 1);
        code |= (variants[2] as usize) << (8 * 0);

        OpCode(code)
    }

    pub fn from_usize(code: usize) -> Self {
        OpCode(code)
    }

    pub fn operation(&self) -> Option<Operation> {
        Operation::from_usize(self.0 >> 8 * 4)
    }

    pub fn variants(&self) -> Option<[Variant; 3]> {
        Some([
            Variant::from_usize((self.0 >> 8 * 2) & 0b1111)?,
            Variant::from_usize((self.0 >> 8 * 1) & 0b1111)?,
            Variant::from_usize((self.0 >> 8 * 0) & 0b1111)?,
        ])
    }

    pub fn split(&self) -> Option<(Operation, [Variant; 3])> {
        Some((self.operation()?, self.variants()?))
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum NativeFunctions {
    Print = 0,
    Exit = 1,
}

impl NativeFunctions {
    pub fn from_string(name: &str) -> Option<NativeFunctions> {
        match name {
            "print" => Some(NativeFunctions::Print),
            "exit" => Some(NativeFunctions::Exit),
            _ => None,
        }
    }

    pub fn from_usize(num: usize) -> Option<NativeFunctions> {
        match num {
            0 => Some(NativeFunctions::Print),
            1 => Some(NativeFunctions::Exit),
            _ => None,
        }
    }

    pub fn to_string(&self) -> Option<&'static str> {
        match self {
            NativeFunctions::Print => Some("print"),
            NativeFunctions::Exit => Some("exit"),
            _ => None,
        }
    }
}
