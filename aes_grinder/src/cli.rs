use clap::Parser as ClapParser;

#[derive(ClapParser)]
#[command(name = "aes_grinder")]
#[command(version = "0.1")]
#[command(about = "Solving a round-reduced AES system", long_about = None)]

pub struct Cli {
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}
