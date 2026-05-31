use clap::{Parser, Subcommand};
use tokio;
mod balance;
mod cache;
mod link;
mod plaid;
mod tui;
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
    Unlink,
    Balance,
    List,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::Link => link::link().await,
        Commands::Unlink => link::unlink().await,
        Commands::Balance => balance::balance().await,
        Commands::List => link::list().await,
    }
}
