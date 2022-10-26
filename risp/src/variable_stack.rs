use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};

#[derive(Debug)]
pub struct VariableStack {
    stack: Vec<HashMap<String, usize>>,
}

impl VariableStack {
    pub fn enter(&mut self) {
        self.stack.push(HashMap::new())
    }
    pub fn leave(&mut self) -> Result<()> {
        self.stack
            .pop()
            .with_context(|| format!("Stack underflowed"))?;
        Ok(())
    }

    pub fn get(&mut self, variable: String) -> Option<usize> {
        for layer in &self.stack {
            if let Some(value) = layer.get(&variable) {
                return Some(*value);
            }
        }

        None
    }

    pub fn set(&mut self, variable: String, current_stack_count: usize) {
        let len = self.stack.len() - 1;
        self.stack[len].insert(variable, current_stack_count);
    }
}
