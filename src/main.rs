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
    /// Create and start new apprentice containers
    Summon {
        /// Names of the apprentices to create
        names: Vec<String>,
        /// Disable system prompt (spawn apprentice without autonomous capabilities)
        #[arg(long)]
        no_system_prompt: bool,
    },
    /// Send a message to an apprentice and get its response
    Tell {
        /// Name of the apprentice to communicate with
        name: String,
        /// The message to send
        message: String,
    },
    /// List all active apprentices
    Ls,
    /// Stop and remove apprentice containers
    Rm {
        /// Names of the apprentices to remove
        names: Vec<String>,
        /// Remove all apprentices
        #[arg(short, long)]
        all: bool,
    },
    /// Show detailed status information for all apprentices
    Ps {
        /// Number of recent chat history lines to show
        #[arg(short, long, default_value = "4")]
        lines: usize,
    },
    /// View and scroll through chat history with an apprentice
    Show {
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
        Commands::Summon {
            names,
            no_system_prompt,
        } => {
            if names.is_empty() {
                println!("❌ No apprentice names provided");
                return Ok(());
            }

            let total = names.len();

            // Print initial messages
            for name in &names {
                println!("🌟 Summoning apprentice {name}...");
            }

            // Execute summons concurrently
            let tasks: Vec<_> = names
                .into_iter()
                .map(|name| {
                    let name_clone = name.clone();
                    let sorcerer = &sorcerer;
                    async move {
                        let agent_mode = !no_system_prompt; // Agent mode is default, --no-system-prompt disables it
                        let result = sorcerer.summon_apprentice(&name, agent_mode).await;
                        (name_clone, result)
                    }
                })
                .collect();

            let results = futures::future::join_all(tasks).await;
            let mut successes = 0;

            // Process results
            for (name, result) in results {
                match result {
                    Ok(_) => {
                        println!("✨ Apprentice {name} has answered your call!");
                        successes += 1;
                    }
                    Err(e) => {
                        error!("Failed to summon apprentice {}: {}", name, e);
                        println!("💀 Failed to summon {name}");
                    }
                }
            }

            if total > 1 {
                println!("\n📊 Summary: {successes}/{total} apprentices summoned successfully");
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
        Commands::Ls => {
            println!("📋 Listing apprentices...");
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
        Commands::Rm { names, all } => {
            let apprentices_to_remove = if all {
                let all_apprentices = sorcerer.list_apprentices().await?;
                if all_apprentices.is_empty() {
                    println!("📭 No apprentices to remove");
                    return Ok(());
                }
                println!("🗑️  Removing all {} apprentices...", all_apprentices.len());
                all_apprentices
            } else {
                if names.is_empty() {
                    println!("❌ No apprentice names provided (use -a for all)");
                    return Ok(());
                }
                names
            };

            let total = apprentices_to_remove.len();
            let mut successes = 0;

            // Print initial messages
            for name in &apprentices_to_remove {
                println!("💀 Removing apprentice {name}...");
            }

            // Execute removals concurrently
            let tasks: Vec<_> = apprentices_to_remove
                .into_iter()
                .map(|name| {
                    let name_clone = name.clone();
                    let sorcerer = &sorcerer;
                    async move {
                        let result = sorcerer.remove_apprentice(&name).await;
                        (name_clone, result)
                    }
                })
                .collect();

            let results = futures::future::join_all(tasks).await;

            // Process results
            for (name, result) in results {
                match result {
                    Ok(_) => {
                        println!("⚰️  Apprentice {name} has been removed!");
                        successes += 1;
                    }
                    Err(e) => {
                        error!("Failed to remove apprentice {}: {}", name, e);
                        println!("⚠️  Failed to remove {name}");
                    }
                }
            }

            if total > 1 {
                println!("\n📊 Summary: {successes}/{total} apprentices removed successfully");
            }
        }
        Commands::Ps { lines } => {
            println!("📊 Overview of apprentices...");
            let statuses = sorcerer.get_all_status().await?;
            if statuses.is_empty() {
                println!("No apprentices found.");
            } else {
                let mut first = true;
                for (name, status) in statuses {
                    if !first {
                        println!(); // Add spacing between apprentices
                    }
                    first = false;

                    // Calculate box width based on apprentice name length
                    let min_width = 45;
                    let name_header = format!(" Apprentice: {name} ");
                    let box_width = min_width.max(name_header.len() + 2);

                    // Draw apprentice info box
                    println!("┌─{}─┐", name_header.pad_to_width(box_width - 4, '─'));
                    println!(
                        "│ State: {:<width$} │",
                        status.state,
                        width = box_width - 11
                    );
                    if !status.last_spell_time.is_empty() {
                        // Parse and format timestamp to be shorter
                        let short_time = if let Ok(dt) =
                            chrono::DateTime::parse_from_rfc3339(&status.last_spell_time)
                        {
                            dt.format("%Y-%m-%d %H:%M:%S").to_string()
                        } else {
                            status.last_spell_time.clone()
                        };
                        let last_msg = format!("Last Message: {short_time}");
                        println!("│ {:<width$} │", last_msg, width = box_width - 4);
                    }
                    println!("└{}┘", "─".repeat(box_width - 2));

                    // Show chat history without boxes
                    match sorcerer.get_chat_history(&name, lines).await {
                        Ok(history) => {
                            if !history.is_empty() {
                                println!("\nRecent Chat History:");
                                for line in history {
                                    print_wrapped_chat_line(&line);
                                }
                            }
                        }
                        Err(e) => {
                            println!("\nCould not retrieve chat history: {e}");
                        }
                    }
                }
            }
        }
        Commands::Show { name, lines } => {
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
                        // Show history directly with proper formatting
                        println!();
                        for line in &history {
                            print_wrapped_chat_line(line);
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

fn print_wrapped_chat_line(line: &str) {
    // Apply formatting to chat lines with bold usernames and mild colors
    for line_part in line.lines() {
        if let Some(colon_pos) = line_part.find(':') {
            let username = &line_part[..colon_pos];
            let message = &line_part[colon_pos..];

            // Apply different colors based on the username
            match username {
                "Sorcerer" => {
                    // Mild blue for Sorcerer
                    println!("\x1b[1;34m{username}\x1b[0m{message}");
                }
                username if username.contains("apprentice-") => {
                    // Mild green for apprentices
                    println!("\x1b[1;32m{username}\x1b[0m{message}");
                }
                _ => {
                    // Default: just bold the username
                    println!("\x1b[1m{username}\x1b[0m{message}");
                }
            }
        } else {
            // No username detected, print as-is
            println!("{line_part}");
        }
    }
}

trait PadToWidth {
    fn pad_to_width(&self, width: usize, pad_char: char) -> String;
}

impl PadToWidth for String {
    fn pad_to_width(&self, width: usize, pad_char: char) -> String {
        if self.len() >= width {
            self.clone()
        } else {
            let padding_needed = width - self.len();
            let left_pad = padding_needed / 2;
            let right_pad = padding_needed - left_pad;
            format!(
                "{}{}{}",
                pad_char.to_string().repeat(left_pad),
                self,
                pad_char.to_string().repeat(right_pad)
            )
        }
    }
}

fn format_chat_line_for_pager(line: &str) -> Vec<String> {
    // Apply formatting to chat lines with bold usernames and mild colors
    line.lines()
        .map(|line_part| {
            if let Some(colon_pos) = line_part.find(':') {
                let username = &line_part[..colon_pos];
                let message = &line_part[colon_pos..];

                // Apply different colors based on the username
                match username {
                    "Sorcerer" => {
                        // Mild blue for Sorcerer
                        format!("\x1b[1;34m{username}\x1b[0m{message}")
                    }
                    username if username.contains("apprentice-") => {
                        // Mild green for apprentices
                        format!("\x1b[1;32m{username}\x1b[0m{message}")
                    }
                    _ => {
                        // Default: just bold the username
                        format!("\x1b[1m{username}\x1b[0m{message}")
                    }
                }
            } else {
                // No username detected, return as-is
                line_part.to_string()
            }
        })
        .collect()
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
                    // Format each line properly before sending to pager
                    let formatted_lines = format_chat_line_for_pager(line);
                    for formatted_line in formatted_lines {
                        writeln!(writer, "{formatted_line}")?;
                    }
                }
                drop(writer); // Close stdin
            }
            let _ = child.wait(); // Wait for pager to finish
        }
        Err(_) => {
            // Fall back to plain output if pager fails
            println!();
            for line in history {
                print_wrapped_chat_line(line);
            }
        }
    }

    Ok(())
}
