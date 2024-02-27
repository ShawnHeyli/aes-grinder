use clap::{Parser as ClapParser, Subcommand};

#[derive(ClapParser)]
#[command(name = "aes_grinder")]
#[command(version = "0.1")]
#[command(about = "Does awesome things", long_about = None)]

pub struct Cli {

    // file contain equation system
    #[arg(short, long, value_name = "FILE")]
    pub equation_system: String,

    /*
    characteristic polynomial
    #[arg(short, long, value_name = "Unsigned Integer")]
    pub characteristic_polynomial: u64,
    */

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// does testing things
    Test {
        /// lists test values
        #[arg(short, long)]
        list: bool,
    },
}

pub fn cli_check (cli: &mut Cli) {
    /* You can check the value provided by positional arguments, 
    or option arguments
    */
    println! ("Equation System input : {}", cli.equation_system);
    //println! ("Characteristic plynomial       : {}", cli.characteristic_polynomial);

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    match cli.debug {
        0 => println! ("Debug mode is off"),
        1 => println! ("Debug mode is on and set to 1"),
        2 => println! ("Debug mode is on and set to 2"),
        3 => println! ("Debug mode is on and set to 3"),
        4 => println! ("Debug mode is on and set to 4"),
        5 => println! ("Debug mode is on and set to 5"),
        _ => {
            cli.debug = 5;
        }
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Test { list }) => {
            if *list {
                println!("Printing testing lists...");
            } else {
                println!("Not printing testing lists...");
            }
        }
        None => {}
    }
}
