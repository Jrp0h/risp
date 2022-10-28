use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use shared::instruction::Variant;

#[derive(Debug, Copy, Clone)]
pub struct Var {
    pub location: usize,
    pub variant: Variant,
}

impl Var {
    pub fn inc(&mut self) {
        self.location += 1;
    }
    pub fn dec(&mut self) {
        self.location -= 1;
    }
}

#[derive(Debug)]
pub struct VariableStack {
    stack: Vec<HashMap<String, Var>>,
    items: Vec<usize>, // FIXME: Better name
}

impl VariableStack {
    pub fn new() -> Self {
        Self {
            stack: vec![],
            items: vec![],
        }
    }
    pub fn enter(&mut self) {
        self.stack.push(HashMap::new());
        self.items.push(0);
    }
    pub fn leave(&mut self) -> Result<()> {
        self.stack
            .pop()
            .with_context(|| format!("Stack underflowed"))?;

        let len = self.items.len() - 1;
        let must_be_popped = self.items[len];
        for _ in 0..must_be_popped {
            self.decrement_relative();
        }
        self.items.pop().unwrap();
        Ok(())
    }

    pub fn increment_relative(&mut self) {
        self.stack.iter_mut().for_each(|layer| {
            layer.iter_mut().for_each(|l| {
                if l.1.variant == Variant::Stack {
                    l.1.inc()
                }
            });
        });
        let len = self.items.len() - 1;
        self.items[len] += 1;
    }

    pub fn decrement_relative(&mut self) {
        self.stack.iter_mut().for_each(|layer| {
            layer.iter_mut().for_each(|l| {
                if l.1.variant == Variant::Stack {
                    l.1.dec()
                }
            });
        });
        let len = self.items.len() - 1;
        self.items[len] -= 1;
    }

    pub fn get(&mut self, variable: String) -> Option<Var> {
        for layer in &self.stack {
            if let Some(value) = layer.get(&variable) {
                return Some(*value);
            }
        }
        None
    }

    // pub fn set(&mut self, variable: String, current_stack_count: usize) -> Result<()> {
    //     let len = self.stack.len() - 1;
    //     self.stack[len].insert(variable, current_stack_count);
    //     todo!("This is wrong, should look up in previous aswell");
    // }

    pub fn create(
        &mut self,
        name: String,
        current_stack_count: usize,
        variant: Variant,
    ) -> Result<()> {
        let len = self.stack.len() - 1;
        if let Some(_) = self.stack[len].get(&name) {
            Err(anyhow::anyhow!("Variable {:?} is already defined", name))
        } else {
            self.stack[len].insert(
                name,
                Var {
                    location: current_stack_count,
                    variant,
                },
            );
            Ok(())
        }
    }
}
