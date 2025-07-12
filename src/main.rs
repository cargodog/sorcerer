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
    /// View and scroll through chat history with an apprentice
    History {
        /// Name of the apprentice to view history for
        name: String,
        /// Number of history lines to show (default: all)
        #[arg(short, long)]
        lines: Option<usize>,
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
        Commands::History { name, lines } => {
            println!("📜 Viewing chat history for apprentice {name}...");

            // Get all history or specified number of lines
            let history_lines = lines.unwrap_or(1000); // Large default to get all history
            match sorcerer.get_chat_history(&name, history_lines).await {
                Ok(history) => {
                    if history.is_empty() {
                        println!("No chat history found for apprentice {name}.");
                        return Ok(());
                    }

                    // If we have many lines and no specific line count was requested, use pager
                    if lines.is_none() && history.len() > 20 {
                        show_history_with_pager(&history)?;
                    } else {
                        // Show history directly
                        println!();
                        for line in &history {
                            println!("{line}");
                        }
                        if history.len() >= history_lines && lines.is_none() {
                            println!("\n(Showing last {history_lines} lines)");
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to get chat history: {}", e);
                    println!("💥 Failed to retrieve chat history for {name}");
                }
            }
        }
    }

    Ok(())
}

fn show_history_with_pager(history: &[String]) -> Result<()> {
    use std::io::{self, Write};
    use std::process::{Command, Stdio};

    // Try to use 'less' first, then fall back to 'more', then plain output
    let pager_cmd = std::env::var("PAGER").unwrap_or_else(|_| {
        if Command::new("less").arg("--version").output().is_ok() {
            "less".to_string()
        } else if Command::new("more").arg("--version").output().is_ok() {
            "more".to_string()
        } else {
            "cat".to_string()
        }
    });

    match Command::new(&pager_cmd)
        .arg("-R") // Support colors in less
        .stdin(Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            if let Some(stdin) = child.stdin.take() {
                let mut writer = io::BufWriter::new(stdin);
                for line in history {
                    writeln!(writer, "{line}")?;
                }
                drop(writer); // Close stdin
            }
            let _ = child.wait(); // Wait for pager to finish
        }
        Err(_) => {
            // Fall back to plain output if pager fails
            println!();
            for line in history {
                println!("{line}");
            }
        }
    }

    Ok(())
}
