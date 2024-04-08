mod algo;
mod cli;
mod matrix;
mod parser;
mod utils;

use clap::Parser as ClapParser;

struct GlobalInfos {
    filename_eq_sys: String,
    sys_name: String,
    polynomial: u16,
}

impl GlobalInfos {
    fn new(filename_eq_sys: String) -> GlobalInfos {
        GlobalInfos {
            filename_eq_sys,
            sys_name: String::new(),
            polynomial: 0x11b,
        }
    }
}

fn main() {
    let mut cli = cli::Cli::parse();
    cli::cli_check(&mut cli);
}
