use crate::{ast::AST, variable_stack::VariableStack};

pub struct CodeGen {
    ast: AST,
    program: Vec<usize>,
    variable_stack: VariableStack,
}
