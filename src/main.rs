use clap::{Parser, Subcommand};
use tokio;
mod balance;
mod cache;
mod daemon;
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
    Encrypt {
        #[arg()]
        token: String,
    },
    Decrypt {
        #[arg()]
        token: String,
    },
}

#[derive(Subcommand, Debug)]
enum PlanSubcommands {
    Create,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::Balance => balance::balance().await,
        Commands::Daemon => daemon::run_daemon(),
        Commands::Link => link::link().await,
        Commands::List => link::list().await,
        Commands::Login => daemon::login(),
        Commands::Ping => daemon::ping(),
        Commands::Plan { command } => match command {
            PlanSubcommands::Create => plan::create().await,
        },
        Commands::Quit => daemon::quit(),
        Commands::Stop => daemon::quit(),
        Commands::Unlink => link::unlink().await,
        Commands::Encrypt { token } => {
            let encrypted = daemon::encrypt_token(token.clone());
            logging::info(format!("response: {}", encrypted).as_str());
        }
        Commands::Decrypt { token } => {
            let encrypted = daemon::decrypt_token(token.clone());
            logging::info(format!("response: {}", encrypted).as_str());
        }
    }
}
