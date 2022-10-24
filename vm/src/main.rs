mod vm;

use shared::instruction::{OpCode, Operation, Variant};

use crate::vm::VM;

macro_rules! variants {
    () => {
        [Variant::None, Variant::None, Variant::None]
    };
    ($var:ident) => {
        [Variant::$var, Variant::None, Variant::None]
    };
    ($var1:ident, $var2:ident) => {
        [Variant::$var1, Variant::$var2, Variant::None]
    };
    ($var1:ident, $var2:ident, $var3:ident) => {
        [Variant::$var1, Variant::$var2, Variant::$var3]
    };
}

macro_rules! op {
    ($op:ident) => {
        OpCode::new(Operation::$op, variants!()).as_usize()
    };
    ($op:ident, $($vars:ident),+) => {
        OpCode::new(Operation::$op, variants!($($vars),*)).as_usize()
    };
}

fn main() {
    let program: Vec<usize> = vec![
        op!(Mov, Register, Direct),
        3,
        70,
        op!(Push, Register),
        3,
        op!(Push, Direct),
        9,
        op!(Add),
        op!(Mov, Register, Stack),
        3,
        0,
        op!(Pop),
    ];

    let program: Vec<usize> = vec![
        // first 2
        op!(Mov, Register, Direct),
        0,
        1,
        op!(Push, Direct),
        0,
        // save last one
        op!(Dup, Stack),
        0,
        op!(Push, Register),
        0,
        op!(Mov, Register, Stack),
        0,
        1,
        op!(Add),
        op!(Jmp, Direct),
        5,
    ];

    for (i, op) in program.iter().enumerate() {
        println!("{}: {} {:#X} {:#b}", i, op, op, op);
    }

    let mut vm = VM::new(program);
    vm.run_max(61);
    vm.dump();
}
