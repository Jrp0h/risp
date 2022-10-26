#[derive(Debug, Clone)]
pub struct TokenSpan {
    pub file: String,
    pub start_line: usize,
    pub start_column: usize,
    end_line: usize,
    end_column: usize,
}

impl TokenSpan {
    pub fn new(
        file: String,
        start_line: usize,
        start_column: usize,
        end_line: usize,
        end_column: usize,
    ) -> Self {
        Self {
            file,
            start_line,
            start_column,
            end_line,
            end_column,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    Identifier,
    Colon,
    Comma,
    Dot,
    LParen,
    RParen,
    LCurly,
    RCurly,
    Dollar,
    Plus,
    Dash,
    Times,
    Slash,
    Equal,
    Percent,
    LessThan,
    GreaterThan,
    Number,
    String,
    EoF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub r#type: TokenType,
    pub value: String,
    pub span: TokenSpan,
}

impl Token {
    pub fn new(r#type: TokenType, span: TokenSpan, value: String) -> Self {
        Self {
            r#type,
            span,
            value,
        }
    }
}
