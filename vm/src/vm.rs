use anyhow::{anyhow, Result};
use shared::instruction::{OpCode, Operation, Variant};

#[derive(Debug, PartialEq, Eq)]
pub enum CmpResult {
    Equal,
    LessThan,
    GreaterThan,
}

pub struct VM {
    program: Vec<usize>,

    pc: usize,
    stack: Vec<usize>,
    register: [usize; 10],
    cmp: CmpResult,
}

impl VM {
    pub fn new(program: Vec<usize>) -> Self {
        Self {
            program,
            pc: 0,
            stack: vec![],
            register: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            cmp: CmpResult::Equal,
        }
    }

    fn advance(&mut self) -> Option<usize> {
        self.pc += 1;
        match self.program.get(self.pc - 1) {
            None => None,
            Some(value) => Some(*value),
        }
    }

    pub fn step(&mut self) -> bool {
        let opcode = OpCode::from_usize(match self.advance() {
            None => return false,
            Some(value) => value,
        });

        match opcode.operation() {
            Some(Operation::Nop) => {}
            Some(Operation::Push) => self.op_push(&opcode),
            Some(Operation::Pop) => self.op_pop(),
            Some(Operation::Add) => self.op_add(),
            Some(Operation::Mult) => self.op_mult(),
            Some(Operation::Sub) => self.op_sub(),
            Some(Operation::Div) => self.op_div(),
            Some(Operation::Mod) => self.op_mod(),
            Some(Operation::Mov) => self.op_mov(&opcode),
            Some(Operation::Dup) => self.op_dup(&opcode),
            Some(Operation::Cmp) => self.op_cmp(&opcode),
            Some(Operation::Jmp) => self.op_jmp(&opcode, Operation::Jmp),
            Some(Operation::JmpEq) => self.op_jmp(&opcode, Operation::JmpEq),
            Some(Operation::JmpNe) => self.op_jmp(&opcode, Operation::JmpNe),
            Some(Operation::JmpGt) => self.op_jmp(&opcode, Operation::JmpGt),
            Some(Operation::JmpLt) => self.op_jmp(&opcode, Operation::JmpLt),
            Some(Operation::JmpGte) => self.op_jmp(&opcode, Operation::JmpGte),
            Some(Operation::JmpLte) => self.op_jmp(&opcode, Operation::JmpLte),
            Some(other) => {
                todo!("Opcode {:?} not implemented", other)
            }
            None => panic!("Invalid opcode {:?}", opcode),
        }

        true
    }

    pub fn run(&mut self) {
        loop {
            if !self.step() {
                break;
            }
        }
    }

    pub fn run_max(&mut self, max: usize) {
        let mut steps = 0;
        loop {
            steps += 1;
            if steps > max {
                break;
            }

            if !self.step() {
                break;
            }
        }
    }

    fn value_from_variant(&self, variant: Variant, value: usize) -> Result<usize> {
        match variant {
            Variant::Direct => Ok(value),
            Variant::Register => Ok(self.register[value]),
            Variant::Stack => Ok(self.stack[self.stack.len() - (value + 1)]),
            other => Err(anyhow!("Can't get value from variant {:?}", other)),
        }
    }

    pub fn dump(&self) {
        println!("Stack:");
        self.dump_stack();

        println!("\nRegisters:");
        self.dump_registers();
        println!("\nCmp Result: {:#?}", self.cmp);
    }

    pub fn dump_stack(&self) {
        if self.stack.len() == 0 {
            println!("Empty")
        }
        for (i, value) in self.stack.iter().enumerate() {
            println!("{}: {}", i, value);
        }
    }

    pub fn dump_registers(&self) {
        for (i, value) in self.register.iter().enumerate() {
            println!("r{}: {}", i, value);
        }
    }

    fn op_add(&mut self) {
        let rhs = self.stack.pop().unwrap();
        let lhs = self.stack.pop().unwrap();
        self.stack.push(lhs + rhs);
    }
    fn op_mult(&mut self) {
        let rhs = self.stack.pop().unwrap();
        let lhs = self.stack.pop().unwrap();
        self.stack.push(lhs * rhs);
    }
    fn op_sub(&mut self) {
        let rhs = self.stack.pop().unwrap();
        let lhs = self.stack.pop().unwrap();
        self.stack.push(lhs - rhs);
    }
    fn op_div(&mut self) {
        let rhs = self.stack.pop().unwrap();
        let lhs = self.stack.pop().unwrap();
        self.stack.push(lhs / rhs);
    }
    fn op_mod(&mut self) {
        let rhs = self.stack.pop().unwrap();
        let lhs = self.stack.pop().unwrap();
        self.stack.push(lhs % rhs);
    }

    fn op_push(&mut self, op: &OpCode) {
        let variant = op.variants().unwrap()[0];
        match variant {
            Variant::Direct => {
                let value = self.advance().unwrap();
                self.stack.push(value)
            }
            Variant::Register => {
                let value = self.advance().unwrap();
                self.stack.push(self.register[value as usize])
            }
            other => panic!("Invalid push variant ({:?})", other),
        }
    }

    fn op_mov(&mut self, op: &OpCode) {
        let where_variant = op.variants().unwrap()[0];
        let where_value = self.advance().unwrap();

        println!(
            "where_value: {:#?}, where_variant: {:#?}",
            where_value, where_variant
        );

        let what_variant = op.variants().unwrap()[1];
        let what_value = self.advance().unwrap();
        println!(
            "what_value: {:#?}, what_variant: {:#?}",
            what_value, what_variant
        );
        let what = self.value_from_variant(what_variant, what_value).unwrap();

        match where_variant {
            Variant::Register => {
                self.register[where_value] = what;
            }
            Variant::Stack => {
                let len = self.stack.len();
                self.stack[len - (where_value + 1)] = what;
            }
            other => panic!("Invalid mov variant ({:?})", other),
        }
    }

    fn op_pop(&mut self) {
        self.stack.pop();
    }

    fn op_jmp(&mut self, op: &OpCode, operation: Operation) {
        let variant = op.variants().unwrap()[0];
        let value = self.advance().unwrap();

        match operation {
            Operation::Jmp => {
                self.pc = self.value_from_variant(variant, value).unwrap();
            }
            Operation::JmpEq => {
                if self.cmp == CmpResult::Equal {
                    self.pc = self.value_from_variant(variant, value).unwrap();
                }
            }
            Operation::JmpNe => {
                if self.cmp != CmpResult::Equal {
                    self.pc = self.value_from_variant(variant, value).unwrap();
                }
            }
            Operation::JmpGt => {
                if self.cmp == CmpResult::GreaterThan {
                    self.pc = self.value_from_variant(variant, value).unwrap();
                }
            }
            Operation::JmpLt => {
                if self.cmp == CmpResult::LessThan {
                    self.pc = self.value_from_variant(variant, value).unwrap();
                }
            }

            Operation::JmpGte => {
                if self.cmp == CmpResult::GreaterThan || self.cmp == CmpResult::Equal {
                    self.pc = self.value_from_variant(variant, value).unwrap();
                }
            }
            Operation::JmpLte => {
                if self.cmp == CmpResult::LessThan || self.cmp == CmpResult::Equal {
                    self.pc = self.value_from_variant(variant, value).unwrap();
                }
            }
            other => panic!("{:?} isn't a jmp operation", other),
        }
    }
    fn op_dup(&mut self, op: &OpCode) {
        let variant = op.variants().unwrap()[0];
        match variant {
            Variant::Stack => {
                let value = self.advance().unwrap();
                if self.stack.len() == 0 {
                    panic!("No elements in stack");
                }
                self.stack.push(self.stack[self.stack.len() - (value + 1)])
            }
            other => panic!("Invalid dup variant ({:?})", other),
        }
    }

    fn op_cmp(&mut self, op: &OpCode) {
        let v = self.advance().unwrap();
        let lhs = self
            .value_from_variant(op.variants().unwrap()[0], v)
            .unwrap();

        let v = self.advance().unwrap();
        let rhs = self
            .value_from_variant(op.variants().unwrap()[1], v)
            .unwrap();

        if lhs > rhs {
            self.cmp = CmpResult::GreaterThan;
        } else if lhs < rhs {
            self.cmp = CmpResult::LessThan;
        } else {
            self.cmp = CmpResult::Equal;
        }
    }
}
