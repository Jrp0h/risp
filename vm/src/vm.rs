use shared::instruction::{OpCode, Operation, Variant};

pub struct VM {
    program: Vec<usize>,

    pc: usize,
    stack: Vec<usize>,
    register: [usize; 10],
}

impl VM {
    pub fn new(program: Vec<usize>) -> Self {
        Self {
            program,
            pc: 0,
            stack: vec![],
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
            Some(Operation::Mov) => self.op_mov(&opcode),
            Some(Operation::Jmp) => self.op_jmp(&opcode),
            Some(Operation::Dup) => self.op_dup(&opcode),
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

    pub fn dump(&self) {
        println!("Stack:");
        self.dump_stack();

        println!("\nRegisters:");
        self.dump_registers();
        println!()
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
        self.stack.push(rhs + lhs);
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

        let what_variant = op.variants().unwrap()[1];

        let what = match what_variant {
            Variant::Direct => self.advance().unwrap(),
            Variant::Register => {
                let value = self.advance().unwrap();
                self.register[value]
            }
            Variant::Stack => {
                let value = self.advance().unwrap();
                if self.stack.len() == 0 {
                    panic!("No elements in stack");
                }
                self.stack[self.stack.len() - (value + 1)]
            }
            other => panic!("Invalid mov variant ({:?})", other),
        };

        match where_variant {
            Variant::Register => {
                self.register[where_value] = what;
            }
            Variant::Stack => {
                self.stack[where_value] = what;
            }
            other => panic!("Invalid mov variant ({:?})", other),
        }
    }

    fn op_pop(&mut self) {
        self.stack.pop();
    }

    fn op_jmp(&mut self, op: &OpCode) {
        let variant = op.variants().unwrap()[0];
        match variant {
            Variant::Direct => {
                self.pc = self.advance().unwrap();
            }
            Variant::Register => {
                let value = self.advance().unwrap();
                self.pc = self.register[value as usize]
            }
            Variant::Stack => {
                let value = self.advance().unwrap();
                if self.stack.len() == 0 {
                    panic!("No elements in stack");
                }
                self.pc = self.stack[self.stack.len() - (value + 1)]
            }
            other => panic!("Invalid jmp variant ({:?})", other),
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
}
