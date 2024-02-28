mod parser;
use clap::Parser as ClapParser;
use parser::Parser;

mod cli;
mod debug;

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
    let mut parser_mod = Parser::new(&globals, cli.debug);

    // NEED TO CATCH MATRIX
    parser_mod
        .parse_system(&mut globals)
        .expect("Error while parsing system");

    //parser::parser (cli.equation_system);
    // Continued program logic goes here...
}
