#[derive(Debug)]
pub enum AST {
    Root(Block), // Only for proc definitions and import
    Block(Block),

    NumberLiteral(NumberLiteral),

    VariableDefinition(VariableDefinition),
    Variable(Identifier),

    FunctionDefinition(FunctionDefinition),

    Call(Call),
}

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Box<AST>>,
}

impl Block {
    pub fn new(statements: Vec<Box<AST>>) -> Self {
        Self { statements }
    }
}

#[derive(Debug)]
pub struct Identifier {
    pub name: String,
}

#[derive(Debug)]
pub struct Call {
    pub id: Identifier,
    pub args: Vec<Box<AST>>,
}

#[derive(Debug)]
pub struct NumberLiteral {
    pub value: usize,
}

#[derive(Debug)]
pub struct VariableDefinition {
    pub id: Identifier,
    pub value: Box<AST>,
}

#[derive(Debug)]
pub struct FunctionDefinition {
    pub id: Identifier,
    pub variables: Vec<Identifier>,
    pub block: Block,
}
