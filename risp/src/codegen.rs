// FIXME: Arguments stack position are incorrect

use anyhow::{anyhow, Context, Result};
use shared::{
    instruction::{NativeFunctions, OpCode, Operation, Variant},
    program::Operand,
    token::TokenType,
};
use std::collections::HashMap;

use crate::{
    ast::{
        BinOp, Block, Call, FromTo, FunctionDefinition, If, Return, VariableDefinition, While, AST,
    },
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

#[derive(Debug)]
struct UnresolvedFunction {
    pub name: String,
    pub location: usize,
}

pub struct CodeGen {
    program: Vec<usize>,
    variable_stack: VariableStack,
    functions: HashMap<String, usize>,
    stack_size: usize,

    unresolved_function: Vec<UnresolvedFunction>,
}

impl CodeGen {
    pub fn new() -> Self {
        Self {
            program: vec![],
            variable_stack: VariableStack::new(),
            functions: HashMap::new(),
            stack_size: 0,
            unresolved_function: vec![],
        }
    }

    fn stack_push(&mut self, variant: Variant, value: usize) -> usize {
        self.program
            .push(OpCode::new(Operation::Push, [variant, Variant::None, Variant::None]).as_usize());
        self.program.push(value);
        self.stack_size += 1;
        self.variable_stack.increment_relative();

        return self.stack_size - 1;
    }

    fn stack_pop(&mut self) {
        self.program.push(op!(Pop));
        self.variable_stack.decrement_relative();
        self.stack_size -= 1;
    }

    // Pop but without popping
    fn stack_lower(&mut self) {
        self.variable_stack.decrement_relative();
        self.stack_size -= 1;
    }

    // push but without pushing
    fn stack_increce(&mut self) {
        self.variable_stack.increment_relative();
        self.stack_size += 1;
    }

    pub fn generate(&mut self, ast: AST) -> Result<(Vec<usize>, usize)> {
        self.variable_stack.enter();
        match ast {
            AST::Root(block) => {
                self.generate_block(&block)?;
            }
            other => return Err(anyhow!("Root must be root, is currently {:?}", other)),
        }

        for func in &self.unresolved_function {
            if let Some(addr) = self.functions.get(&func.name) {
                self.program[func.location] = *addr;
            } else {
                return Err(anyhow!("Unknown function {}", func.name));
            }
        }

        let entry = self
            .functions
            .get("main")
            .with_context(|| anyhow!("main function not defined"))?;
        self.variable_stack.enter();

        Ok((self.program.clone(), *entry))
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

            if let Some(v) = self.functions.get(&call.id.name) {
                self.program.push(*v);
            } else {
                self.unresolved_function.push(UnresolvedFunction {
                    name: call.id.name.clone(),
                    location: self.program.len(),
                });
                self.program.push(0);
            }
        }
        // Silently push value from return
        self.stack_increce();

        // Pop all args
        for _ in &call.args {
            self.program.push(op!(Swap));
            self.stack_pop();
        }

        Ok(())
    }

    pub fn has_call(&self, ast: &AST) -> bool {
        match ast {
            AST::NumberLiteral(_) => false,
            AST::Call(_) => true,
            AST::FunctionDefinition(_) => false,
            AST::VariableDefinition(var) => self.has_call(&var.value),
            AST::VariableSet(var) => self.has_call(&var.value),
            AST::Variable(_) => false,
            AST::BinOp(binop) => self.has_call(&binop.lhs) || self.has_call(&binop.rhs),
            AST::Return(ret) => self.has_call(&ret.value),
            AST::If(ef) => self.has_call(&ef.cond),
            AST::While(wile) => self.has_call(&wile.cond),
            other => todo!("Implement {:?}", other),
        }
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
                // FiXME: This should be pushed
                return Ok(Some(Operand::new(v.location, v.variant)));
            }
            AST::BinOp(binop) => {
                self.generate_binop(binop)?;
                return Ok(Some(Operand::new(0, Variant::Stack)));
            }
            AST::Return(ret) => self.generate_return(ret)?,
            AST::If(ef) => self.generate_if(ef)?,
            AST::While(wile) => self.generate_while(wile)?,
            AST::FromTo(ft) => self.generate_from_to(ft)?,
            other => todo!("Implement {:?}", other),
        }

        Ok(None)
    }

    pub fn generate_block(&mut self, block: &Block) -> Result<()> {
        self.variable_stack.enter();
        for stmt in &block.statements {
            self.generate_statement(&(*stmt))?;
        }
        self.variable_stack.leave()?;

        Ok(())
    }

    pub fn generate_function(&mut self, definition: &FunctionDefinition) -> Result<()> {
        // self.variable_stack.enter();
        // TODO: Validate that the function isnt already defined
        self.functions
            .insert(definition.id.name.clone(), self.program.len());

        for (i, var) in definition.variables.iter().enumerate() {
            self.variable_stack
                .create(var.name.clone(), i, Variant::Stack)?;
        }

        self.generate_block(&definition.block)?;
        // self.variable_stack.leave()?;
        Ok(())
    }

    pub fn generate_variable_definition(&mut self, definition: &VariableDefinition) -> Result<()> {
        let value = self.generate_statement(&(*definition.value))?;
        let value = value.with_context(|| anyhow!("Variable definition must be a value"))?;

        self.stack_push(value.variant, value.value);
        self.variable_stack.create(
            definition.id.name.clone(),
            // self.stack_size,
            // Variant::StackAbsoulute,
            0,
            Variant::Stack,
        )?;
        Ok(())
    }

    pub fn generate_set_variable(&mut self, definition: &VariableDefinition) -> Result<()> {
        let value = self.generate_statement(&(*definition.value))?;
        let value = value.with_context(|| anyhow!("Set Variable must be a value"))?;

        // This must be after since the stack might change durring statement generation of the
        // value
        let variable = self
            .variable_stack
            .get(definition.id.name.clone())
            .with_context(|| anyhow!("Unknown variable {}", definition.id.name))?;

        self.program.push(
            OpCode::new(
                Operation::Mov,
                [Variant::Stack, value.variant, Variant::None],
                // [Variant::StackAbsoulute, value.variant, Variant::None],
            )
            .as_usize(),
        );

        self.program.push(variable.location);
        // if value.variant == Variant::Stack {
        //     self.program.push(self.stack_size - value.value - 1);
        // } else {
        self.program.push(value.value);
        // }

        self.stack_pop(); // remove value from

        Ok(())
    }

    pub fn push_if_not_last_on_stack(&mut self, ast: &AST, operand: Operand) {
        // FIXME: This is just a test
        match ast {
            AST::Variable(_) => {
                self.stack_push(operand.variant, operand.value);
            }
            _ => {
                if operand.variant == Variant::Stack && operand.value == 0 {
                    return;
                }
                self.stack_push(operand.variant, operand.value);
            }
        }
    }

    pub fn generate_binop(&mut self, binop: &BinOp) -> Result<()> {
        let value = self.generate_statement(&(*binop.lhs))?;
        let lhs = value.with_context(|| anyhow!("LHS must evaluate to a value"))?;

        let value = self.generate_statement(&(*binop.rhs))?;
        let rhs = value.with_context(|| anyhow!("RHS must evaluate to a value"))?;

        // self.stack_push(lhs.variant, lhs.value);
        // self.stack_push(rhs.variant, rhs.value);
        self.push_if_not_last_on_stack(&binop.lhs, lhs);
        self.push_if_not_last_on_stack(&binop.rhs, rhs);

        match binop.op {
            TokenType::Plus => self.program.push(op!(Add)),
            TokenType::Dash => self.program.push(op!(Sub)),
            TokenType::Times => self.program.push(op!(Mult)),
            TokenType::Slash => self.program.push(op!(Div)),
            TokenType::Percent => self.program.push(op!(Mod)),
            TokenType::Equal => self.program.push(op!(CmpEq)),
            TokenType::LessThan => self.program.push(op!(CmpLt)),
            TokenType::GreaterThan => self.program.push(op!(CmpGt)),
            other => return Err(anyhow!("{:?} isn't a valid binary operation", other)),
        }

        self.stack_lower(); // all binops removes one from the stack

        // self.program.push(op!(Mov, Register, Stack));
        // self.program.push(0);
        // self.program.push(0);
        // self.stack_pop();

        Ok(())
    }

    pub fn generate_return(&mut self, ret: &Return) -> Result<()> {
        let value = self.generate_statement(&(*ret.value))?;
        let value = value.with_context(|| anyhow!("return must evaluate to a value"))?;
        // self.stack_push(value.variant, value.value);
        self.push_if_not_last_on_stack(&ret.value, value);
        self.program.push(op!(Ret));
        Ok(())
    }

    pub fn generate_if(&mut self, ef: &If) -> Result<()> {
        // self.variable_stack.enter();
        let value = self.generate_statement(&(*ef.cond))?;
        let cond = value.with_context(|| anyhow!("condition must evaluate to a value"))?;
        self.push_if_not_last_on_stack(&ef.cond, cond);
        // self.stack_push(cond.variant, cond.value);

        self.program.push(op!(Not));
        self.program.push(op!(JmpIf, Direct));
        self.program.push(0);
        self.stack_lower(); // jmp removed condition;
        let jmp_to_else_addr = self.program.len() - 1;

        self.generate_block(&ef.then)?;
        self.program.push(op!(Jmp, Direct));
        self.program.push(10);
        let jmp_to_end_addr = self.program.len() - 1;

        self.program[jmp_to_else_addr] = self.program.len();
        if let Some(else_block) = &ef.r#else {
            self.generate_block(else_block)?;
        }

        self.program[jmp_to_end_addr] = self.program.len() - 1;
        // self.variable_stack.leave()?;
        Ok(())
    }

    pub fn generate_from_to(&mut self, ft: &FromTo) -> Result<()> {
        // self.variable_stack.enter();
        let value = self.generate_statement(&(*ft.start))?;
        let start = value.with_context(|| anyhow!("start must evaluate to a value"))?;

        let value = self.generate_statement(&(*ft.finish))?;
        let finish = value.with_context(|| anyhow!("finish must evaluate to a value"))?;

        // push start
        let var = self.stack_push(start.variant, start.value); // current var

        let loop_start = self.program.len();
        // push current and finish
        self.stack_push(Variant::StackAbsoulute, var);
        self.stack_push(finish.variant, finish.value);

        //cmp
        self.program.push(op!(CmpLt));
        self.program.push(op!(Not));
        self.program.push(op!(JmpIf, Direct));
        self.program.push(0);
        let end_addr = self.program.len() - 1;

        // Generate action
        self.generate_block(&ft.block)?;

        // add
        self.stack_push(Variant::StackAbsoulute, var);
        self.stack_push(Variant::Direct, 1);
        self.program.push(op!(Add));

        self.program.push(op!(Mov, StackAbsoulute, Stack));
        self.program.push(var);
        self.program.push(0);
        self.stack_pop(); // cmp
        self.stack_pop(); // negated

        // Jump back
        self.program.push(op!(Jmp, Direct));
        self.program.push(loop_start);

        self.program[end_addr] = self.program.len();
        self.stack_pop(); // current
        self.stack_pop(); // start

        // self.variable_stack.leave()?;
        Ok(())
    }

    pub fn generate_while(&mut self, wile: &While) -> Result<()> {
        // self.variable_stack.enter();
        let start_addr = self.program.len();

        let value = self.generate_statement(&(*wile.cond))?;
        let cond = value.with_context(|| anyhow!("condition must evaluate to a value"))?;
        self.push_if_not_last_on_stack(&wile.cond, cond);
        // self.stack_push(cond.variant, cond.value);

        self.program.push(op!(Not));
        self.program.push(op!(JmpIf, Direct));
        self.stack_lower(); // jmp removed condition;
        self.program.push(0);
        let jmp_to_end_addr = self.program.len() - 1;

        self.generate_block(&wile.then)?;
        self.program.push(op!(Jmp, Direct));
        self.program.push(start_addr);

        self.program[jmp_to_end_addr] = self.program.len();

        // self.variable_stack.leave()?;
        Ok(())
    }
}
