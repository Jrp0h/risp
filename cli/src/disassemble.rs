use std::{fs::File, io::Write};

use shared::program::ProgramParser;

pub struct DisassembleArgs {
    pub input_path: String,
    pub output_path: Option<String>,
}

pub fn disassemble(args: DisassembleArgs) {
    let program = shared::fileformat::FileFormat::from_file(args.input_path)
        .unwrap()
        .program;
    let program = ProgramParser::new(program).parse().unwrap();

    if let Some(output_path) = args.output_path {
        let mut f = File::open(output_path).unwrap();
        let text = program.to_string().bytes().collect::<Vec<u8>>();
        f.write_all(&text).unwrap()
    } else {
        println!("{}", program.to_string());
    }
}
