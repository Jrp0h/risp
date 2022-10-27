use asm::assembler::Assembler;
use risp::{codegen::CodeGen, parser::Parser};
use shared::{lexer::Lexer, program::ProgramParser, token::Token};
use vm::vm::VM;

pub struct RunArgs {
    pub filepath: String,
    pub max_instructions: Option<usize>,
    pub dump: bool,
}

pub fn run(args: RunArgs) {
    let program;
    let mut entry = 0;

    if args.filepath.ends_with(".rasm") {
        // Assembly
        let lexer = Lexer::new_from_path(args.filepath.to_string());
        let mut asm = Assembler::new(lexer).unwrap();
        program = asm.assemble().unwrap();
    } else if args.filepath.ends_with(".risp") {
        // Lisp
        let lexer = Lexer::new_from_path(args.filepath.to_string());
        let ast = Parser::parse(lexer).unwrap();
        let bytecode = CodeGen::new().generate(ast).unwrap();
        program = bytecode.0.clone();
        entry = bytecode.1;
    } else {
        // Bin
        program = shared::fileformat::FileFormat::from_file(args.filepath)
            .unwrap()
            .program;
    }

    let mut vm = VM::new(program, entry);

    if let Some(max) = args.max_instructions {
        vm.run_max(max);
    } else {
        vm.run();
    }

    if args.dump {
        vm.dump();
    }
}
