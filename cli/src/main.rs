use clap::{Parser, Subcommand};
use run::RunArgs;
use shared::program::ProgramParser;
mod compile;
mod disassemble;
mod run;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        file: String,

        #[arg(short = 'm', long)]
        max_instructions: Option<usize>,

        #[arg(short = 'd', long)]
        dump: bool,
    },
    Compile {
        input_path: String,

        #[arg(short = 'o', long)]
        output_path: Option<String>,

        #[arg(long)]
        ast: bool,
    },
    Disassemble {
        input_path: String,

        #[arg(short = 'o', long)]
        output_path: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run {
            file,
            max_instructions,
            dump,
        } => {
            run::run(RunArgs {
                filepath: file.to_string(),
                max_instructions: *max_instructions,
                dump: *dump,
            });
        }
        Commands::Compile {
            input_path,
            output_path,
            ast,
        } => {
            compile::compile(compile::CompileArgs {
                input_path: input_path.to_string(),
                output_path: output_path.clone(),
                ast: *ast,
            });
        }
        Commands::Disassemble {
            input_path,
            output_path,
        } => disassemble::disassemble(disassemble::DisassembleArgs {
            input_path: input_path.to_string(),
            output_path: output_path.clone(),
        }),
    }
}
