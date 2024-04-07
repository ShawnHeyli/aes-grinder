mod algo;
mod cli;
mod matrix;
mod parser;

use clap::Parser as ClapParser;

use crate::{matrix::Matrix, parser::Parser};

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

    // NEED TO CATCH MATRIX
    let matrix = parser_mod
        .parse_system(&mut globals)
        .expect("Error while parsing system");
    let mut matrix2 = matrix::Matrix::from(matrix);
    matrix2.set_vars_map(parser_mod.vars_map);
    
    println!("{}", matrix2);
    matrix2.display_var_map();

}
