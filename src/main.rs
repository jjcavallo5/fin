use clap::{Parser, Subcommand};
use tokio;
mod balance;
mod cache;
mod db;
mod entity;
mod link;
mod logging;
mod plaid;
mod plan;
mod tui;

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
    Plan {
        #[command(subcommand)]
        command: PlanSubcommands,
    },
}

#[derive(Subcommand, Debug)]
enum PlanSubcommands {
    Create,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let db = db::get_db().await;

    match &args.command {
        Commands::Link => link::link().await,
        Commands::Unlink => link::unlink().await,
        Commands::Balance => balance::balance().await,
        Commands::List => link::list().await,
        Commands::Plan { command } => match command {
            PlanSubcommands::Create => plan::create().await,
        },
    }
}
