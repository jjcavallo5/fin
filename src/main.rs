use clap::{Parser, Subcommand};
use tokio;
mod balance;
mod daemon;
mod db;
mod entity;
mod environment;
mod link;
mod logging;
mod migration;
mod money;
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
    Balance,
    Daemon,
    Link,
    Login,
    List,
    Ping,
    Plan {
        #[command(subcommand)]
        command: PlanSubcommands,
    },
    Quit,
    Stop,
    Unlink,

    #[command(alias = "nw")]
    NetWorth,
}

#[derive(Subcommand, Debug)]
enum PlanSubcommands {
    Create,
    Execute { plan_id: i32 },
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::Balance => balance::balance().await,
        Commands::Daemon => daemon::run_daemon().await,
        Commands::Link => link::link().await,
        Commands::List => link::list().await,
        Commands::Login => daemon::login(),
        Commands::Ping => daemon::ping(),
        Commands::Plan { command } => match command {
            PlanSubcommands::Create => plan::create().await,
            PlanSubcommands::Execute { plan_id } => plan::execute(*plan_id).await,
        },
        Commands::Quit => daemon::quit(),
        Commands::Stop => daemon::quit(),
        Commands::Unlink => link::unlink().await,
        Commands::NetWorth => balance::net_worth().await,
    }
}
