use anyhow::Result;
use clap::{Parser, Subcommand};
use std::env;
use tracing::{info, warn};

mod daemon;
mod ssh_protocol;
mod telegram;
mod proxy;

#[derive(Parser)]
#[command(name = "git-sentry")]
#[command(about = "SSH Agent proxy with out-of-band 2FA via Telegram", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Git-Sentry daemon
    Daemon {
        /// Socket path (default: /tmp/git-sentry.sock or Windows equivalent)
        #[arg(short, long)]
        socket: Option<String>,

        /// Telegram bot token
        #[arg(short, long, env = "GIT_SENTRY_BOT_TOKEN")]
        bot_token: String,

        /// Telegram chat ID for notifications
        #[arg(short, long, env = "GIT_SENTRY_CHAT_ID")]
        chat_id: i64,

        /// Path to real SSH agent socket
        #[arg(short, long, env = "SSH_AUTH_SOCK")]
        ssh_auth_sock: Option<String>,

        /// Approval timeout in seconds
        #[arg(short, long, default_value = "300")]
        timeout: u64,
    },

    /// Setup: Initialize environment
    Setup {
        /// Telegram bot token
        #[arg(short, long)]
        bot_token: String,

        /// Telegram chat ID
        #[arg(short, long)]
        chat_id: i64,
    },

    /// Test connection to Telegram
    Test {
        /// Telegram bot token
        #[arg(short, long, env = "GIT_SENTRY_BOT_TOKEN")]
        bot_token: String,

        /// Telegram chat ID
        #[arg(short, long, env = "GIT_SENTRY_CHAT_ID")]
        chat_id: i64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Daemon {
            socket,
            bot_token,
            chat_id,
            ssh_auth_sock,
            timeout,
        } => {
            info!("Starting Git-Sentry daemon");
            daemon::run(daemon::DaemonConfig {
                socket_path: socket.or_else(|| env::var("GIT_SENTRY_SOCKET").ok()),
                bot_token,
                chat_id,
                ssh_auth_sock,
                approval_timeout_secs: timeout,
            })
            .await?;
        }

        Commands::Setup {
            bot_token,
            chat_id,
        } => {
            info!("Setting up Git-Sentry");
            println!(
                "Add the following to your ~/.bashrc or ~/.zshrc:\n\n\
                 export GIT_SENTRY_BOT_TOKEN=\"{}\"\n\
                 export GIT_SENTRY_CHAT_ID=\"{}\"\n\
                 export SSH_AUTH_SOCK=\"/tmp/git-sentry.sock\"\n\n\
                 Then start the daemon with: git-sentry daemon",
                bot_token, chat_id
            );
        }

        Commands::Test {
            bot_token,
            chat_id,
        } => {
            info!("Testing Telegram connection");
            telegram::send_message(
                &bot_token,
                chat_id,
                "🔐 Git-Sentry test message - connection successful!",
            )
            .await?;
            info!("✓ Telegram connection test passed");
        }
    }

    Ok(())
}
