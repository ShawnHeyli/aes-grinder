mod algo;
mod cli;
mod exhaustive_search;
mod matrix;
mod parser;
mod utils;

use std::fs::read_dir;

use crate::cli::Cli;
use clap::Parser as ClapParser;
use dialoguer::FuzzySelect;
use exhaustive_search::{exhaustive_search, random_search, Search};
use strum::IntoEnumIterator;

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
    // Sets the verbosity flag
    Cli::parse();

    let selection = FuzzySelect::new()
        .with_prompt("What type of search ?")
        .items(
            &Search::iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
        )
        .interact()
        .unwrap();
    let search: Search = Search::iter().nth(selection).unwrap();

    // files <directory>/<filename>
    let files = &["test", "equation_system"]
        .into_iter()
        .flat_map(|dir| {
            read_dir(dir).unwrap().map(move |file| {
                format!(
                    "{}/{}",
                    dir,
                    file.unwrap().file_name().into_string().unwrap()
                )
            })
        })
        .collect::<Vec<String>>();
    let selection: usize = FuzzySelect::new()
        .with_prompt("With which system ?")
        .items(
            // files in test/ and equation_system/
            files,
        )
        .interact()
        .unwrap();
    let system: &str = files.get(selection).unwrap();

    let mut globals: GlobalInfos = GlobalInfos::new(system.to_owned());
    let mut parser_mod = parser::Parser::new(&globals);

    let matrix = parser_mod
        .parse_system(&mut globals)
        .expect("Error while parsing system");
    let mut matrix = matrix;
    matrix.set_vars_map(parser_mod.vars_map);

    println!("{}", matrix);
    matrix.drop_linear_variables();
    println!("{}", matrix);

    match search {
        Search::Exhaustive => {
            exhaustive_search(&mut matrix, 6)
                .iter()
                .next()
                .unwrap()
                .to_dot_debug("/tmp/algo.dot", &matrix)
                .unwrap();
        }
        Search::Random => {
            random_search(&mut matrix)
                .to_dot_debug("/tmp/algo.dot", &matrix)
                .unwrap();
        }
    }
}
