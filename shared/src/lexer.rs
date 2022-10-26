use crate::token::{Token, TokenSpan, TokenType};
use std::{char, fs};

#[derive(Debug)]
pub struct Lexer {
    i: usize,
    data: Vec<char>,

    current_line: usize,
    current_column: usize,

    filepath: String,

    has_eof: bool,
}
impl Lexer {
    pub fn new_from_path(filepath: String) -> Self {
        let err = format!("Failed to open {filepath}");
        let data = fs::read_to_string(filepath.clone())
            .expect(&err)
            .chars()
            .into_iter()
            .collect();

        Self {
            i: 0,
            data,
            current_line: 1,
            current_column: 0,
            filepath,
            has_eof: false,
        }
    }

    fn check_newline(&mut self) {
        let c = self.data[self.i];
        if c == '\n' {
            self.current_line += 1;
            self.current_column = 0;
        }
    }

    fn skip_comment(&mut self) {
        while self.data[self.i] != '\n' {
            self.advance();
            self.check_newline();
        }
    }

    fn get_char_token(&mut self) -> Option<Token> {
        let c = self.data[self.i] as char;
        let span = TokenSpan::new(
            self.filepath.clone(),
            self.current_line,
            self.current_column,
            self.current_line,
            self.current_column + 1,
        );

        match c {
            ':' => Some(Token::new(TokenType::Colon, span, c.to_string())),
            '=' => Some(Token::new(TokenType::Equal, span, c.to_string())),
            '{' => Some(Token::new(TokenType::LCurly, span, c.to_string())),
            '}' => Some(Token::new(TokenType::RCurly, span, c.to_string())),
            '(' => Some(Token::new(TokenType::LParen, span, c.to_string())),
            ')' => Some(Token::new(TokenType::RParen, span, c.to_string())),
            ',' => Some(Token::new(TokenType::Comma, span, c.to_string())),
            '.' => Some(Token::new(TokenType::Dot, span, c.to_string())),
            '$' => Some(Token::new(TokenType::Dollar, span, c.to_string())),
            '+' => Some(Token::new(TokenType::Plus, span, c.to_string())),
            '-' => Some(Token::new(TokenType::Dash, span, c.to_string())),
            '*' => Some(Token::new(TokenType::Times, span, c.to_string())),
            '/' => Some(Token::new(TokenType::Slash, span, c.to_string())),
            '<' => Some(Token::new(TokenType::LessThan, span, c.to_string())),
            '>' => Some(Token::new(TokenType::GreaterThan, span, c.to_string())),
            '%' => Some(Token::new(TokenType::Percent, span, c.to_string())),
            _ => None,
        }
    }

    fn capture_string(&mut self) -> Token {
        let mut string = String::new();
        let start_line = self.current_line;
        let start_col = self.current_column;

        loop {
            self.advance();
            if self.current_as_char() == '"' {
                let t = Token::new(
                    TokenType::String,
                    TokenSpan::new(
                        self.filepath.clone(),
                        start_line,
                        start_col,
                        self.current_line,
                        self.current_column,
                    ),
                    string,
                );
                self.advance();
                return t;
            }
            if self.current_as_char() == '\\' {
                let c = self.data[self.i + 1] as char;
                match c {
                    '\\' => string.push('\\'),
                    'n' => string.push('\n'),
                    't' => string.push('\t'),
                    'r' => string.push('\r'),
                    _ => panic!("Unknown escape sequence {}", c),
                }
                self.advance();
                continue;
            }
            string.push(self.current_as_char());
        }
    }

    fn capture_number(&mut self) -> Token {
        let mut number = String::new();
        let start_line = self.current_line;
        let start_col = self.current_column;

        // TODO: Add support for hex and binary numbers
        while self.current_as_char().is_numeric() {
            number.push(self.current_as_char());
            self.advance();
        }

        return Token::new(
            TokenType::Number,
            TokenSpan::new(
                self.filepath.clone(),
                start_line,
                start_col,
                self.current_line,
                self.current_column,
            ),
            number,
        );
    }

    fn capture_identifier(&mut self) -> Token {
        let mut string = String::new();
        let start_line = self.current_line;
        let start_col = self.current_column;

        while self.current_as_char().is_alphanumeric()
            || self.current_as_char() == '_'
            || self.current_as_char() == '-'
        {
            string.push(self.current_as_char());
            self.advance();
        }

        return Token::new(
            TokenType::Identifier,
            TokenSpan::new(
                self.filepath.clone(),
                start_line,
                start_col,
                self.current_line,
                self.current_column,
            ),
            string,
        );
    }

    fn current_as_char(&mut self) -> char {
        self.data[self.i] as char
    }

    fn peek_as_char(&mut self) -> char {
        self.data[self.i + 1] as char
    }

    fn advance(&mut self) {
        self.check_newline();
        self.i += 1;
        self.current_column += 1;
    }

    fn skip_whitespace(&mut self) {
        while self.current_as_char().is_whitespace() {
            if self.i == self.data.len() - 1 {
                return;
            }
            self.advance();
        }
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        while self.i < self.data.len() {
            self.skip_whitespace();

            if self.current_as_char() == ';' {
                self.skip_comment();
            }

            // if self.current_as_char() == '/' && self.peek_as_char() == '/' {
            //     self.skip_comment();
            //     continue;
            // }

            if let Some(token) = self.get_char_token() {
                self.advance();
                return Some(token);
            }

            if self.current_as_char() == '"' {
                return Some(self.capture_string());
            }

            if self.current_as_char().is_numeric() {
                return Some(self.capture_number());
            }

            if self.current_as_char().is_alphabetic() {
                return Some(self.capture_identifier());
            }

            // TODO: Handle unknown token
            self.advance();
        }

        if self.has_eof {
            None
        } else {
            self.has_eof = true;
            Some(Token::new(
                TokenType::EoF,
                TokenSpan::new(
                    self.filepath.clone(),
                    self.current_line,
                    self.current_column,
                    self.current_line,
                    self.current_column,
                ),
                "EOF".to_string(),
            ))
        }
    }
}
