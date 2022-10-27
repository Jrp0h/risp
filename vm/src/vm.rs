use anyhow::{anyhow, Result};
use shared::instruction::{NativeFunctions, OpCode, Operation, Variant};

pub struct VM {
    program: Vec<usize>,

    pc: usize,
    stack: Vec<usize>,
    call_stack: Vec<usize>,
    register: [usize; 10],
}

impl VM {
    pub fn new(program: Vec<usize>, entry: usize) -> Self {
        Self {
            program,
            pc: entry,
            stack: vec![],
            call_stack: vec![],
            register: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
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
            Some(Operation::Jmp) => self.op_jmp(&opcode, Operation::Jmp),
            Some(Operation::JmpIf) => self.op_jmp(&opcode, Operation::JmpIf),
            Some(Operation::CmpEq) => self.op_cmp(&opcode, Operation::CmpEq),
            Some(Operation::CmpNe) => self.op_cmp(&opcode, Operation::CmpNe),
            Some(Operation::CmpGt) => self.op_cmp(&opcode, Operation::CmpGt),
            Some(Operation::CmpLt) => self.op_cmp(&opcode, Operation::CmpLt),
            Some(Operation::CmpGte) => self.op_cmp(&opcode, Operation::CmpGte),
            Some(Operation::CmpLte) => self.op_cmp(&opcode, Operation::CmpLte),
            Some(Operation::Call) => return self.op_call(&opcode),
            Some(Operation::Ret) => self.op_ret(),
            Some(Operation::Not) => self.op_not(),
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
            Variant::StackRelative => Ok(self.stack[value]),
            other => Err(anyhow!("Can't get value from variant {:?}", other)),
        }
    }

    pub fn dump(&self) {
        println!("Stack:");
        self.dump_stack();

        println!("\nRegisters:");
        self.dump_registers();
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
            Variant::Stack => {
                let value = self.advance().unwrap();
                let len = self.stack.len();
                self.stack.push(self.stack[len - (value + 1)]);
            }
            Variant::StackRelative => {
                let value = self.advance().unwrap();
                self.stack.push(self.stack[value as usize])
            }
            other => panic!("Invalid push variant ({:?})", other),
        }
    }

    fn op_mov(&mut self, op: &OpCode) {
        let where_variant = op.variants().unwrap()[0];
        let where_value = self.advance().unwrap();

        let what_variant = op.variants().unwrap()[1];
        let what_value = self.advance().unwrap();

        let what = self.value_from_variant(what_variant, what_value).unwrap();

        match where_variant {
            Variant::Register => {
                self.register[where_value] = what;
            }
            Variant::Stack => {
                let len = self.stack.len();
                self.stack[len - (where_value + 1)] = what;
            }
            Variant::StackRelative => {
                self.stack[where_value] = what;
            }
            other => panic!("Invalid mov variant ({:?})", other),
        }
    }

    fn op_pop(&mut self) {
        self.stack.pop();
    }

    fn op_cmp(&mut self, op: &OpCode, operation: Operation) {
        // let v = self.advance().unwrap();
        // let lhs = self
        //     .value_from_variant(op.variants().unwrap()[0], v)
        //     .unwrap();

        // let v = self.advance().unwrap();
        // let rhs = self
        //     .value_from_variant(op.variants().unwrap()[1], v)
        //     .unwrap();

        let rhs = self.stack.pop().unwrap();
        let lhs = self.stack.pop().unwrap();

        match operation {
            Operation::CmpEq => {
                self.stack.push((lhs == rhs) as usize);
            }
            Operation::CmpNe => {
                self.stack.push((lhs != rhs) as usize);
            }
            Operation::CmpGt => {
                self.stack.push((lhs > rhs) as usize);
            }
            Operation::CmpLt => {
                self.stack.push((lhs < rhs) as usize);
            }
            Operation::CmpGte => {
                self.stack.push((lhs >= rhs) as usize);
            }
            Operation::CmpLte => {
                self.stack.push((lhs <= rhs) as usize);
            }
            other => panic!("{:?} isn't a cmp operation", other),
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

    fn op_call(&mut self, op: &OpCode) -> bool {
        let value = self.advance().unwrap();
        let variant = op.variants().unwrap()[0];
        match variant {
            Variant::Direct => {
                self.call_stack.push(self.pc + 1);
                self.pc = value;
            }
            Variant::Native => {
                if value == NativeFunctions::Print as usize {
                    println!("{}", self.stack[self.stack.len() - 1]);
                }
                if value == NativeFunctions::Exit as usize {
                    return false;
                }
            }
            _ => panic!("Invalid call variant {:?}", variant),
        }

        return true;
    }

    fn op_ret(&mut self) {
        self.pc = self.call_stack.pop().unwrap();
    }

    fn op_not(&mut self) {
        let res = self.stack.pop().unwrap() == 0;
        self.stack.push(res as usize);
    }

    fn op_jmp(&mut self, op: &OpCode, operation: Operation) {
        let variant = op.variants().unwrap()[0];
        let value = self.advance().unwrap();

        match operation {
            Operation::Jmp => {
                self.pc = self.value_from_variant(variant, value).unwrap();
            }
            Operation::JmpIf => {
                let cond = self.stack[self.stack.len() - 1];
                if cond != 0 {
                    let addr = self.value_from_variant(variant, value).unwrap();
                    self.pc = addr;
                }
            }
            _ => panic!("Invalid jmp variant {:?}", variant),
        }
    }
}
