use crate::instruction::{NativeFunctions, OpCode, Operation, Variant};
use anyhow::{anyhow, Context, Result};

#[derive(Clone, Copy, Debug)]
pub struct Operand {
    pub value: usize,
    pub variant: Variant,
}

impl Operand {
    pub fn new(value: usize, variant: Variant) -> Self {
        Self { value, variant }
    }

    pub fn format(&self) -> String {
        match self.variant {
            Variant::Stack => format!("s({})", self.value),
            Variant::Register => format!("r({})", self.value),
            Variant::Direct => format!("{}", self.value),
            Variant::Native => format!(
                "${}",
                NativeFunctions::from_usize(self.value)
                    .unwrap()
                    .to_string()
                    .unwrap()
            ), // TODO: Look up native function name from number
            Variant::None | Variant::Indirect => "".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Action {
    pub operation: Operation,
    pub operands: Vec<Operand>,
}

impl Action {
    pub fn new(operation: Operation, operands: Vec<Operand>) -> Self {
        Self {
            operation,
            operands,
        }
    }

    pub fn format(&self) -> String {
        format!(
            "{} {}",
            self.operation.to_asm(),
            self.operands
                .iter()
                .map(|o| o.format())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Clone, Debug)]
pub struct Program {
    pub actions: Vec<Action>,
}

impl Program {
    pub fn new(actions: Vec<Action>) -> Self {
        Self { actions }
    }

    pub fn to_string(&self) -> String {
        let mut assembly = "".to_string();
        for action in &self.actions {
            assembly.push_str(format!("  {}\n", action.format()).as_str());
        }

        assembly
    }
}

#[derive(Clone, Debug)]
pub struct ProgramParser {
    bytes: Vec<usize>,
    pc: usize,
    actions: Vec<Action>,
}

impl ProgramParser {
    pub fn new(bytes: Vec<usize>) -> Self {
        Self {
            bytes,
            pc: 0,
            actions: vec![],
        }
    }

    pub fn parse(&mut self) -> Result<Program> {
        while self.pc < self.bytes.len() {
            let action = self.step()?;
            self.actions.push(action);
        }

        Ok(Program::new(self.actions.clone()))
    }

    pub fn step(&mut self) -> Result<Action> {
        let opcode = OpCode::from_usize(match self.advance() {
            None => return Err(anyhow!("djawjdakwd")),
            Some(value) => value,
        });

        match opcode.operation() {
            Some(Operation::Nop) => self.collect_zero(&opcode),
            Some(Operation::Push) => self.collect_one(&opcode),
            Some(Operation::Pop) => self.collect_zero(&opcode),
            Some(Operation::Add) => self.collect_zero(&opcode),
            Some(Operation::Mult) => self.collect_zero(&opcode),
            Some(Operation::Sub) => self.collect_zero(&opcode),
            Some(Operation::Div) => self.collect_zero(&opcode),
            Some(Operation::Mov) => self.collect_two(&opcode),
            Some(Operation::Jmp) => self.collect_one(&opcode),
            Some(Operation::JmpEq) => self.collect_one(&opcode),
            Some(Operation::JmpNe) => self.collect_one(&opcode),
            Some(Operation::JmpGt) => self.collect_one(&opcode),
            Some(Operation::JmpLt) => self.collect_one(&opcode),
            Some(Operation::JmpGte) => self.collect_one(&opcode),
            Some(Operation::JmpLte) => self.collect_one(&opcode),
            Some(Operation::Dup) => self.collect_one(&opcode),
            Some(Operation::Cmp) => self.collect_two(&opcode),
            Some(Operation::Call) => self.collect_one(&opcode),
            Some(Operation::Ret) => self.collect_zero(&opcode),
            Some(other) => {
                todo!("Opcode {:?} not implemented", other)
            }
            None => panic!("Invalid opcode {:?}", opcode),
        }
    }

    fn advance(&mut self) -> Option<usize> {
        self.pc += 1;
        match self.bytes.get(self.pc - 1) {
            None => None,
            Some(value) => Some(*value),
        }
    }

    fn collect_zero(&mut self, op: &OpCode) -> Result<Action> {
        Ok(Action::new(op.operation().unwrap(), vec![]))
    }

    fn collect_one(&mut self, op: &OpCode) -> Result<Action> {
        let variants = op
            .variants()
            .with_context(|| format!("Failed to collect variants"))?;

        Ok(Action::new(
            op.operation().unwrap(),
            vec![Operand::new(self.advance().unwrap(), variants[0])],
        ))
    }
    fn collect_two(&mut self, op: &OpCode) -> Result<Action> {
        let variants = op
            .variants()
            .with_context(|| format!("Failed to collect variants"))?;

        Ok(Action::new(
            op.operation().unwrap(),
            vec![
                Operand::new(self.advance().unwrap(), variants[0]),
                Operand::new(self.advance().unwrap(), variants[1]),
            ],
        ))
    }
}
