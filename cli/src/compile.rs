use asm::assembler::Assembler;
use shared::{fileformat::FileFormat, lexer::Lexer};

pub struct CompileArgs {
    pub input_path: String,
    pub output_path: Option<String>,
}

pub fn compile(args: CompileArgs) {
    let lexer = Lexer::new_from_path(args.input_path.to_string());
    let mut asm = Assembler::new(lexer).unwrap();
    let program = asm.assemble().unwrap();
    let format = FileFormat::new(program);
    format
        .write_to_file(args.output_path.unwrap_or("a.out".to_string()))
        .unwrap(); // TODO: output should be
                   // input but with .bin instead
}
