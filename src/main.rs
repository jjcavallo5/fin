use clap::{Parser, Subcommand};
use tokio;
mod balance;
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
        nonce: String,
        #[arg()]
        ciphertext: String,
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
        Commands::Daemon => daemon::run_daemon().await,
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
            if let Some((nonce, ciphertext)) = daemon::encrypt_token(token.clone()) {
                logging::info(format!("nonce: {}", nonce).as_str());
                logging::info(format!("ciphertext: {}", ciphertext).as_str());
            }
        }
        Commands::Decrypt { nonce, ciphertext } => {
            if let Some(decrypted) = daemon::decrypt_token(nonce.clone(), ciphertext.clone()) {
                logging::info(format!("response: {}", decrypted).as_str());
            }
        }
    }
}
