use anyhow::{anyhow, Context, Result};
use shared::{
    instruction::{NativeFunctions, OpCode, Operation, Variant},
    program::Operand,
    token::TokenType,
};
use std::collections::HashMap;

use crate::{
    ast::{BinOp, Block, Call, FunctionDefinition, VariableDefinition, AST},
    variable_stack::VariableStack,
};
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

pub struct CodeGen {
    program: Vec<usize>,
    variable_stack: VariableStack,
    functions: HashMap<String, usize>,
    stack_size: usize,
}

impl CodeGen {
    pub fn new() -> Self {
        Self {
            program: vec![],
            variable_stack: VariableStack::new(),
            functions: HashMap::new(),
            stack_size: 0,
        }
    }

    fn stack_push(&mut self, variant: Variant, value: usize) {
        self.program
            .push(OpCode::new(Operation::Push, [variant, Variant::None, Variant::None]).as_usize());
        self.program.push(value);
        self.stack_size += 1;
    }

    fn stack_pop(&mut self) {
        self.program.push(op!(Pop));
        self.stack_size -= 1;
    }

    pub fn generate(&mut self, ast: AST) -> Result<Vec<usize>> {
        match ast {
            AST::Root(block) => {
                self.generate_block(&block)?;
            }
            other => return Err(anyhow!("Root must be root, is currently {:?}", other)),
        }

        Ok(self.program.clone())
    }

    pub fn generate_call(&mut self, call: &Call) -> Result<()> {
        // Push all args onto stack
        for arg in &call.args {
            let value = self.generate_statement(&(*arg))?;
            let value = value
                .with_context(|| anyhow!("Function call arguments must evaluate to a value"))?;
            self.stack_push(value.variant, value.value);
        }

        if let Some(func) = NativeFunctions::from_string(&call.id.name) {
            self.program.push(op!(Call, Native));
            self.program.push(func as usize);
        } else {
            self.program.push(op!(Call, Direct));
            let func = self
                .functions
                .get(&call.id.name)
                .expect("TODO: Implement unresolved functions");
            self.program.push(*func);
        }

        // Pop the args
        for _ in &call.args {
            self.stack_pop();
        }

        Ok(())
    }

    pub fn generate_statement(&mut self, statement: &AST) -> Result<Option<Operand>> {
        match statement {
            AST::Block(block) => self.generate_block(block)?,
            AST::NumberLiteral(num) => {
                // self.stack_push(Variant::Direct, num.value);
                return Ok(Some(Operand::new(num.value, Variant::Direct)));
            }
            AST::Call(call) => {
                self.generate_call(call)?;
                return Ok(Some(Operand::new(0, Variant::Stack)));
            }
            AST::FunctionDefinition(func) => self.generate_function(func)?,
            AST::VariableDefinition(var) => self.generate_variable_definition(var)?,
            AST::VariableSet(var) => self.generate_set_variable(var)?,
            AST::Variable(var) => {
                let v = self
                    .variable_stack
                    .get(var.name.clone())
                    .with_context(|| format!("Unknown variable {}", var.name))?;
                return Ok(Some(Operand::new(v, Variant::Stack)));
            }
            AST::BinOp(binop) => {
                self.generate_binop(binop)?;
                return Ok(Some(Operand::new(0, Variant::Register)));
            }
            other => todo!("Implement {:?}", other),
        }

        Ok(None)
    }

    pub fn generate_block(&mut self, block: &Block) -> Result<()> {
        for stmt in &block.statements {
            self.generate_statement(&(*stmt))?;
        }

        Ok(())
    }

    pub fn generate_function(&mut self, definition: &FunctionDefinition) -> Result<()> {
        self.variable_stack.enter();
        // TODO: Validate that the function isnt already defined
        self.functions
            .insert(definition.id.name.clone(), self.program.len() + 1);

        for (i, var) in definition.variables.iter().enumerate() {
            self.variable_stack
                .create(var.name.clone(), self.stack_size - i)?;
        }

        self.generate_block(&definition.block)?;
        self.variable_stack.leave()?;
        Ok(())
    }

    pub fn generate_variable_definition(&mut self, definition: &VariableDefinition) -> Result<()> {
        self.variable_stack
            .create(definition.id.name.clone(), self.stack_size)?;
        let value = self.generate_statement(&(*definition.value))?;
        let value = value.with_context(|| anyhow!("Variable definition must be a value"))?;

        self.stack_push(value.variant, value.value);
        Ok(())
    }

    pub fn generate_set_variable(&mut self, definition: &VariableDefinition) -> Result<()> {
        let variable = self
            .variable_stack
            .get(definition.id.name.clone())
            .with_context(|| anyhow!("Unknown variable {}", definition.id.name))?;

        let value = self.generate_statement(&(*definition.value))?;
        let value = value.with_context(|| anyhow!("Set Variable must be a value"))?;

        self.program.push(
            OpCode::new(
                Operation::Mov,
                [Variant::Stack, value.variant, Variant::None],
            )
            .as_usize(),
        );

        self.program.push(self.stack_size - variable - 1);
        if value.variant == Variant::Stack {
            self.program.push(self.stack_size - value.value - 1);
        } else {
            self.program.push(value.value);
        }

        Ok(())
    }

    pub fn generate_binop(&mut self, binop: &BinOp) -> Result<()> {
        let value = self.generate_statement(&(*binop.lhs))?;
        let lhs = value.with_context(|| anyhow!("LHS must evaluate to a value"))?;

        let value = self.generate_statement(&(*binop.rhs))?;
        let rhs = value.with_context(|| anyhow!("RHS must evaluate to a value"))?;

        self.stack_push(lhs.variant, lhs.value);
        self.stack_push(rhs.variant, rhs.value);

        match binop.op {
            TokenType::Plus => self.program.push(op!(Add)),
            TokenType::Dash => self.program.push(op!(Sub)),
            TokenType::Times => self.program.push(op!(Mult)),
            TokenType::Slash => self.program.push(op!(Div)),
            TokenType::Percent => self.program.push(op!(Mod)),
            TokenType::Equal | TokenType::LessThan | TokenType::GreaterThan => {
                todo!("BinOp codegen comparison")
            }
            other => return Err(anyhow!("{:?} isn't a valid binary operation", other)),
        }

        self.stack_size -= 1; // all binops removes one from the stack

        self.program.push(op!(Mov, Register, Stack));
        self.program.push(0);
        self.program.push(0);
        self.stack_pop();

        Ok(())
    }
}
