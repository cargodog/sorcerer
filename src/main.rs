mod config;
mod sorcerer;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "srcrr")]
#[command(about = "🧙‍♂️ The Sorcerer - Command apprentices to do your bidding")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create and start a new apprentice container
    Summon {
        /// Name of the apprentice to create
        name: String,
    },
    /// Send a message to an apprentice and get its response
    Tell {
        /// Name of the apprentice to communicate with
        name: String,
        /// The message to send
        message: String,
    },
    /// List all active apprentices
    Scry,
    /// Stop and remove an apprentice container
    Banish {
        /// Name of the apprentice to remove
        name: String,
    },
    /// Show detailed status information for all apprentices
    Grimoire {
        /// Number of recent chat history lines to show
        #[arg(short, long, default_value = "4")]
        lines: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "sorcerer=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();
    let mut sorcerer = sorcerer::Sorcerer::new().await?;

    match cli.command {
        Commands::Summon { name } => {
            println!("🌟 Summoning apprentice {name}...");
            match sorcerer.summon_apprentice(&name).await {
                Ok(_) => {
                    println!("✨ Apprentice {name} has answered your call!");
                }
                Err(e) => {
                    error!("Failed to summon apprentice: {}", e);
                    println!("💀 The summoning failed");
                }
            }
        }
        Commands::Tell { name, message } => {
            println!("📜 Sending message to apprentice {name}...");
            match sorcerer.cast_spell(&name, &message).await {
                Ok(response) => {
                    println!("🔮 The apprentice responds:");
                    println!("{response}");
                }
                Err(e) => {
                    error!("Message sending failed: {}", e);
                    println!("💥 The message failed");
                }
            }
        }
        Commands::Scry => {
            println!("👁️  Scrying for apprentices...");
            println!();
            let apprentices = sorcerer.list_apprentices().await?;
            if apprentices.is_empty() {
                println!("The realm is empty - no apprentices found.");
            } else {
                for apprentice in apprentices {
                    println!("🧙 {apprentice}");
                }
            }
        }
        Commands::Banish { name } => {
            println!("🌪️  Banishing apprentice {name}...");
            match sorcerer.banish_apprentice(&name).await {
                Ok(_) => {
                    println!("💨 Apprentice {name} has been banished!");
                }
                Err(e) => {
                    error!("Failed to banish apprentice: {}", e);
                    println!("⚠️  Banishment failed");
                }
            }
        }
        Commands::Grimoire { lines } => {
            println!("📖 Consulting the grimoire...");
            let statuses = sorcerer.get_all_status().await?;
            if statuses.is_empty() {
                println!("The grimoire is empty - no apprentices found.");
            } else {
                for (name, status) in statuses {
                    println!("═══════════════════════════════");
                    println!("Apprentice: {name}");
                    println!("State: {}", status.state);
                    if !status.last_spell_time.is_empty() {
                        println!("Last Message: {}", status.last_spell_time);
                    }

                    // Show chat history
                    match sorcerer.get_chat_history(&name, lines).await {
                        Ok(history) => {
                            if !history.is_empty() {
                                println!("Recent Chat History:");
                                for line in history {
                                    let mut lines_iter = line.lines();
                                    if let Some(first_line) = lines_iter.next() {
                                        println!("  {first_line}");
                                        // For multi-line messages, indent subsequent lines to align with the message content
                                        for subsequent_line in lines_iter {
                                            // Find the position after the speaker name (e.g., "Sorcerer: " or "AppName: ")
                                            let indent_size =
                                                if let Some(colon_pos) = first_line.find(": ") {
                                                    colon_pos + 2 + 2 // colon + space + initial 2-space indent
                                                } else {
                                                    2 // fallback to basic indent
                                                };
                                            println!(
                                                "{}{}",
                                                " ".repeat(indent_size),
                                                subsequent_line
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("  Could not retrieve chat history: {e}");
                        }
                    }
                }
                println!("═══════════════════════════════");
            }
        }
    }

    Ok(())
}
