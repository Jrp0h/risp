use std::iter::Peekable;

use crate::ast::{Block, Call, FunctionDefinition, Identifier, VariableDefinition, AST};
use shared::lexer::Lexer;
use shared::token::{Token, TokenType};

use anyhow::{anyhow, Context, Result};

macro_rules! error_at {
    ($loc:expr, $msg:expr,  $($items:expr),*) => {{
        let msg = format!($msg, $($items),*);
        let msg = format!("{}, at {}:{}:{}", msg, $loc.file, $loc.start_line, $loc.start_column);
        anyhow::anyhow!(msg)
    }
    };
    ($loc:expr, $msg:expr) => {{
        let msg = format!("{}, at {}:{}:{}", $msg, $loc.file, $loc.start_line, $loc.start_column);
        anyhow::anyhow!(msg)
    }
    };
}
pub struct Parser {
    lexer: Peekable<Lexer>,
    current: Token,
}

impl Parser {
    pub fn parse(mut lexer: Lexer) -> Result<AST> {
        let token = lexer.next().expect("Ran out of tokens");

        let mut parser = Parser {
            lexer: lexer.peekable(),
            current: token,
        };

        parser.parse_root()
    }
    fn advance(&mut self) -> Token {
        let current = self.current.clone();
        self.current = self.lexer.next().expect("Ran out of tokens");
        current
    }

    fn peek(&mut self) -> Result<Token> {
        Ok(self
            .lexer
            .peek()
            .with_context(|| error_at!(self.current.span, "Unexpected eof"))?
            .clone())
    }

    fn eat(&mut self, expected: TokenType) -> Result<Token> {
        println!("Expected: {:?} got {:?}", expected, self.current.r#type);
        if self.current.r#type == expected {
            let old = self.current.clone();
            self.advance();
            Ok(old)
        } else {
            Err(error_at!(
                self.current.span,
                "Expected {:?} but got {:?} {:?}",
                expected,
                self.current.r#type,
                self.current.value
            ))
        }
    }

    fn parse_binop(&mut self) -> Result<AST> {
        todo!("Implement")
    }

    fn parse_number_variable_or_statement(&mut self) -> Result<AST> {
        match self.current.r#type {
            TokenType::Number => {
                let value = self.eat(TokenType::Number)?;

                Ok(AST::NumberLiteral(crate::ast::NumberLiteral {
                    value: value.value.parse::<usize>()?,
                }))
            }
            TokenType::Dollar => {
                self.eat(TokenType::Dollar)?;
                let id = self.eat(TokenType::Identifier)?;

                Ok(AST::Variable(Identifier { name: id.value }))
            }
            TokenType::LParen => self.parse_statement(),
            _ => todo!("figure out what to do here"),
        }
    }

    fn parse_function_call(&mut self) -> Result<AST> {
        let name = self.eat(TokenType::Identifier)?; // ex print
        let mut args = vec![];
        while self.current.r#type != TokenType::RParen {
            args.push(Box::new(self.parse_number_variable_or_statement()?));
        }
        Ok(AST::Call(Call {
            name: Identifier { name: name.value },
            args,
        }))
    }

    fn parse_block(&mut self) -> Result<AST> {
        self.eat(TokenType::LCurly)?;
        let statements = self.parse_statements()?;
        self.eat(TokenType::RCurly)?;
        Ok(AST::Block(Block { statements }))
    }

    fn parse_function_definition(&mut self) -> Result<AST> {
        let mut variables: Vec<Identifier> = vec![];

        self.eat(TokenType::Identifier)?; // defun
        let name = self.eat(TokenType::Identifier)?; // ex main

        // $arg1 $arg2
        while self.current.r#type == TokenType::Dollar {
            self.eat(TokenType::Dollar)?;
            let id = self.eat(TokenType::Identifier)?;
            variables.push(Identifier { name: id.value })
        }

        let block = self.parse_block()?;

        Ok(AST::FunctionDefinition(FunctionDefinition {
            name: Identifier { name: name.value },
            variables,
            block: Box::new(block),
        }))
    }

    fn parse_keyword(&mut self) -> Result<AST> {
        match self.current.value.as_str() {
            "defun" => self.parse_function_definition(),
            // "defvar" => todo!("Implement"),
            "print" => self.parse_function_call(),
            "exit" => self.parse_function_call(),
            _ => todo!("Not implemented yet"),
        }
    }

    fn parse_statement(&mut self) -> Result<AST> {
        self.eat(TokenType::LParen)?;

        let statement = match self.current.r#type {
            TokenType::Plus
            | TokenType::Dash
            | TokenType::Slash
            | TokenType::Times
            | TokenType::Equal
            | TokenType::GreaterThan
            | TokenType::LessThan
            | TokenType::Percent => self.parse_binop()?,
            TokenType::Identifier => self.parse_keyword()?,
            TokenType::Number => AST::NumberLiteral(crate::ast::NumberLiteral {
                value: self.current.value.parse::<usize>()?,
            }),
            _ => todo!("Implement "),
        };
        self.eat(TokenType::RParen)?;
        Ok(statement)
    }

    fn parse_statements(&mut self) -> Result<Vec<Box<AST>>> {
        let mut statements = vec![];

        while self.current.r#type != TokenType::EoF
            && self.current.r#type != TokenType::RParen
            && self.current.r#type != TokenType::RCurly
        {
            statements.push(Box::new(self.parse_statement()?));
        }

        Ok(statements)
    }

    fn parse_root(&mut self) -> Result<AST> {
        let mut statements = vec![];

        while self.current.r#type != TokenType::EoF {
            statements.push(Box::new(self.parse_statement()?));
        }

        Ok(AST::Root(Block { statements }))
    }
}
