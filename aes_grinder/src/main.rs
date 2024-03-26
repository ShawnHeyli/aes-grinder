mod algo;
mod cli;
mod matrix;
mod parser;

use clap::Parser as ClapParser;
use matrix::Matrix;
use parser::Parser;

struct GlobalInfos {
    filename_eq_sys: String,
    sys_name: String,
}

impl GlobalInfos {
    fn new(filename_eq_sys: String) -> GlobalInfos {
        GlobalInfos {
            filename_eq_sys,
            sys_name: String::new(),
        }
    }
}

fn main() {
    let mut cli = cli::Cli::parse();
    cli::cli_check(&mut cli);

    let mut globals: GlobalInfos = GlobalInfos::new(cli.equation_system);
    let mut parser_mod = Parser::new(&globals);

    let matrix = Matrix::new(2, 2);
    println!("{}", matrix);
    let matrix_just_parsed = parser_mod.parse_system(&mut globals).unwrap();
    let matrix:Matrix = matrix_just_parsed.into();

    print!("{:?}", matrix);

    // parser::parser (cli.equation_system);
    // Continued program logic goes here...
}
