use std::{fs::File, io::Write};

use asm::assembler::Assembler;
use risp::{codegen::CodeGen, parser::Parser};
use shared::{fileformat::FileFormat, lexer::Lexer, program::ProgramParser};

pub struct CompileArgs {
    pub input_path: String,
    pub output_path: Option<String>,
    pub ast: bool,
    pub asm: bool,
}

pub fn compile(args: CompileArgs) {
    let output = args.output_path.unwrap_or("a.bin".to_string());

    if args.input_path.ends_with(".rasm") {
        let lexer = Lexer::new_from_path(args.input_path.to_string());
        let mut asm = Assembler::new(lexer).unwrap();
        let program = asm.assemble().unwrap();

        let format = FileFormat::new(program);
        format.write_to_file(output).unwrap(); // TODO: output should be
    } else if args.input_path.ends_with(".risp") {
        // Lisp
        let lexer = Lexer::new_from_path(args.input_path.to_string());
        let ast = Parser::parse(lexer).unwrap();

        if args.ast {
            // --ast  then print the ast
            println!("{:#?}", ast);
            return;
        }

        let program = CodeGen::new().generate(ast).unwrap();

        if args.asm {
            let program = ProgramParser::new(program.0.clone()).parse().unwrap();
            println!("{}", program.to_string());
        }

        // Output as rasm
        if output.ends_with(".rasm") {
            let mut f = File::create(output).unwrap();
            let program = ProgramParser::new(program.0).parse().unwrap();
            let text = program.to_string().bytes().collect::<Vec<u8>>();
            f.write_all(&text).unwrap()
        } else {
            let format = FileFormat::new(program.0);
            format.write_to_file(output).unwrap();
        }
    } else {
        panic!("Unknown file format");
    }
}
