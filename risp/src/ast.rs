use shared::token::TokenType;

#[derive(Debug)]
pub enum AST {
    Root(Block), // Only for proc definitions and import
    Block(Block),

    NumberLiteral(NumberLiteral),

    VariableDefinition(VariableDefinition),
    VariableSet(VariableDefinition),
    Variable(Identifier),

    FunctionDefinition(FunctionDefinition),

    Call(Call),

    BinOp(BinOp),
    Return(Return),

    If(If),
    FromTo(FromTo),
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

#[derive(Debug)]
pub struct BinOp {
    pub lhs: Box<AST>,
    pub rhs: Box<AST>,
    pub op: TokenType, // TODO: Make this its own thing
}

#[derive(Debug)]
pub struct Return {
    pub value: Box<AST>,
}

#[derive(Debug)]
pub struct If {
    pub cond: Box<AST>,
    pub then: Block,
    pub r#else: Option<Block>,
}

#[derive(Debug)]
pub struct FromTo {
    pub start: Box<AST>,
    pub finish: Box<AST>,
    pub block: Block,
}
