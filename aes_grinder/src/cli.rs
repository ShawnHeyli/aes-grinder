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
    // Quiet or verbose mode
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

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

pub fn cli_check(cli: &mut Cli) {
    /* You can check the value provided by positional arguments,
    or option arguments
    */
    println!("Equation System input : {}", cli.equation_system);
    //println! ("Characteristic plynomial       : {}", cli.characteristic_polynomial);

    // Initialize the program-wide logger
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

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
