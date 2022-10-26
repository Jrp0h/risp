use std::{collections::HashMap, iter::Peekable};

use anyhow::{anyhow, Context, Result};
use shared::instruction::{OpCode, Operation, Variant};
use shared::{
    lexer::Lexer,
    token::{Token, TokenSpan, TokenType},
};

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

#[derive(Debug)]
pub struct Assembler {
    lexer: Peekable<Lexer>,
    current: Token,

    labels: HashMap<String, usize>,
    unresolved_labels: Vec<UnresolvedLabel>,
    program: Vec<usize>,
}

impl Assembler {
    pub fn new(mut lexer: Lexer) -> Result<Self> {
        let current = lexer.next().with_context(|| format!("Lexer was empty"))?;

        Ok(Self {
            lexer: lexer.peekable(),
            current,
            labels: HashMap::new(),
            unresolved_labels: Vec::new(),
            program: vec![],
        })
    }

    fn advance(&mut self) -> Result<Token> {
        // println!("advancing from {:#?}", self.current);
        let old = self.current.clone();
        self.current = self
            .lexer
            .next()
            .with_context(|| error_at!(self.current.span, "Ran out of tokens"))?;
        Ok(old)
    }

    fn eat(&mut self, expected: TokenType) -> Result<Token> {
        if self.current.r#type == expected {
            self.advance()
        } else {
            Err(error_at!(
                self.current.span,
                "Expected {:?} but got {:?}",
                expected,
                self.current.r#type
            ))
        }
    }

    pub fn assemble(&mut self) -> Result<Vec<usize>> {
        while self.current.r#type != TokenType::EoF {
            let mut instructions = self.next()?;
            self.program.append(&mut instructions);
        }

        self.resolve_labels()?;

        Ok(self.program.clone())
    }

    fn next(&mut self) -> Result<Vec<usize>> {
        match self.current.r#type {
            TokenType::Dot => {
                self.eat(TokenType::Dot)?;
                let label = self.eat(TokenType::Identifier)?;
                self.eat(TokenType::Colon)?;
                self.labels.insert(label.value, self.program.len());
                return Ok(vec![]);
            }
            TokenType::Identifier => {
                return self.handle_instruction();
            }
            _ => return Err(anyhow!("dwajdkaw")),
        }
    }

    fn handle_instruction(&mut self) -> Result<Vec<usize>> {
        let instruction = self.eat(TokenType::Identifier)?;

        match instruction.value.as_str() {
            "mov" => self.handle_mov(),
            "push" => self.handle_push(),
            "dup" => self.handle_dup(),
            "add" => self.handle_math(Operation::Add),
            "sub" => self.handle_math(Operation::Sub),
            "mult" => self.handle_math(Operation::Mult),
            "div" => self.handle_math(Operation::Div),
            "jmp" => self.handle_jmp(),
            other => Err(error_at!(
                self.current.span,
                "Unknown instruction {}",
                other
            )),
        }
    }

    fn capture_operand(&mut self) -> Result<Operand> {
        let current = self.advance()?;

        match current.r#type {
            TokenType::Number => {
                let num = current.value.parse::<usize>().with_context(|| {
                    error_at!(self.current.span, "{} is not a valid number", current.value)
                })?;
                Ok(Operand::Direct(num))
            }
            TokenType::Identifier => {
                let id = current;
                self.eat(TokenType::LParen)?;
                let num = self
                    .eat(TokenType::Number)?
                    .value
                    .parse::<usize>()
                    .with_context(|| format!("{} is not a valid number", id.value))?;
                self.eat(TokenType::RParen)?;
                match id.value.as_str() {
                    "s" => Ok(Operand::Stack(num)),
                    "r" => Ok(Operand::Register(num)),
                    other => Err(error_at!(
                        self.current.span,
                        "Unknown operation '{:?}'",
                        other
                    )),
                }
            }
            TokenType::Dot => {
                let label = self.eat(TokenType::Identifier)?;
                Ok(Operand::Label(label.value))
            }
            other => Err(error_at!(
                self.current.span,
                "Operand cant start with {:?}",
                other
            )),
        }
    }

    fn handle_mov(&mut self) -> Result<Vec<usize>> {
        let first = self.capture_operand()?;
        self.eat(TokenType::Comma)?;
        let second = self.capture_operand()?;
        let variants = [first.as_variant()?, second.as_variant()?, Variant::None];

        Ok(vec![
            OpCode::new(Operation::Mov, variants).as_usize(),
            first.as_usize()?,
            second.as_usize()?,
        ])
    }

    fn handle_push(&mut self) -> Result<Vec<usize>> {
        let operand = self.capture_operand()?;
        let variants = [operand.as_variant()?, Variant::None, Variant::None];

        Ok(vec![
            OpCode::new(Operation::Push, variants).as_usize(),
            operand.as_usize()?,
        ])
    }

    fn handle_dup(&mut self) -> Result<Vec<usize>> {
        let operand = self.capture_operand()?;
        let variants = [operand.as_variant()?, Variant::None, Variant::None];

        Ok(vec![
            OpCode::new(Operation::Dup, variants).as_usize(),
            operand.as_usize()?,
        ])
    }

    fn handle_math(&mut self, op: Operation) -> Result<Vec<usize>> {
        let variants = [Variant::None, Variant::None, Variant::None];
        Ok(vec![OpCode::new(op, variants).as_usize()])
    }

    fn handle_jmp(&mut self) -> Result<Vec<usize>> {
        let operand = self.capture_operand()?;
        match operand {
            Operand::Label(label) => {
                if let Some(pos) = self.labels.get(&label) {
                    let variants = [Variant::Direct, Variant::None, Variant::None];

                    Ok(vec![OpCode::new(Operation::Jmp, variants).as_usize(), *pos])
                } else {
                    self.unresolved_labels.push(UnresolvedLabel {
                        label,
                        location: self.program.len() + 1, // +1 for +0 is where the operation goes,
                        // not the operand
                        span: self.current.span.clone(),
                    });
                    let variants = [Variant::Direct, Variant::None, Variant::None];
                    Ok(vec![OpCode::new(Operation::Jmp, variants).as_usize(), 17])
                }
            }
            _ => {
                let variants = [operand.as_variant()?, Variant::None, Variant::None];

                Ok(vec![
                    OpCode::new(Operation::Jmp, variants).as_usize(),
                    operand.as_usize()?,
                ])
            }
        }
    }

    pub fn resolve_labels(&mut self) -> Result<()> {
        for label in &self.unresolved_labels {
            let label_loc = self
                .labels
                .get(&label.label)
                .with_context(|| error_at!(label.span, "Couldn't find label '{}'", label.label))?;
            self.program[label.location] = *label_loc;
        }

        Ok(())
    }
}

enum Operand {
    Register(usize),
    Direct(usize),
    Stack(usize),
    Label(String),
}

impl Operand {
    pub fn as_variant(&self) -> Result<Variant> {
        match self {
            Operand::Register(_) => Ok(Variant::Register),
            Operand::Direct(_) => Ok(Variant::Direct),
            Operand::Stack(_) => Ok(Variant::Stack),
            _ => Err(anyhow!("Operand cant be a variant")),
        }
    }

    pub fn as_usize(&self) -> Result<usize> {
        match self {
            Operand::Register(v) => Ok(*v),
            Operand::Direct(v) => Ok(*v),
            Operand::Stack(v) => Ok(*v),
            _ => Err(anyhow!("Operand cant be a usize")),
        }
    }

    pub fn as_string(&self) -> Result<String> {
        match self {
            Operand::Label(s) => Ok(s.clone()),
            _ => Err(anyhow!("Operand cant be a string")),
        }
    }
}

#[derive(Debug)]
struct UnresolvedLabel {
    pub label: String,
    pub location: usize,
    pub span: TokenSpan,
}
