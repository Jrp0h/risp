use lexer::Lexer;
use token::Token;

use crate::assembler::Assembler;

mod assembler;
mod lexer;
mod token;

fn main() {
    let lexer = Lexer::new_from_path("test_files/fib.asm".to_string());
    // let tokens: Vec<Token> = lexer.collect();
    // println!("{:#?}", tokens);
    let mut asm = Assembler::new(lexer).unwrap();
    let program = asm.assemble().unwrap();

    for (i, op) in program.iter().enumerate() {
        println!("{}: {} {:#X} {:#b}", i, op, op, op);
    }
}
