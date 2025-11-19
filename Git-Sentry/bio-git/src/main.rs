use anyhow::Result;
use clap::Parser;
use std::io::BufRead;
use std::io::BufReader;

mod credential_helper;
mod biometric;
mod keyring_mgr;

#[derive(Parser)]
#[command(name = "git-credential-bio")]
#[command(about = "Git credential helper with biometric verification")]
struct Args {
    #[command(subcommand)]
    operation: Operation,
}

#[derive(clap::Subcommand)]
enum Operation {
    /// Retrieve credentials
    Get,
    /// Store credentials
    Approve,
    /// Reject/clear credentials
    Reject,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // Read credential attributes from stdin
    let stdin = std::io::stdin();
    let reader = BufReader::new(stdin.lock());
    let attrs = credential_helper::parse_credential_request(reader)?;

    match args.operation {
        Operation::Get => {
            credential_helper::handle_get(&attrs).await?;
        }
        Operation::Approve => {
            credential_helper::handle_approve(&attrs).await?;
        }
        Operation::Reject => {
            credential_helper::handle_reject(&attrs).await?;
        }
    }

    Ok(())
}
