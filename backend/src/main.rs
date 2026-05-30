use clap::{Parser, Subcommand};
mod link;
mod utils;

#[derive(Parser, Debug)]
#[command(name = "fin")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Link,
}

fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::Link => link::link(),
    }
}
