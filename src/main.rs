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
                                    print_wrapped_chat_line(&line);
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

fn get_terminal_width() -> usize {
    if let Some(size) = termsize::get() {
        size.cols as usize
    } else {
        80 // fallback to 80 columns
    }
}

fn wrap_text(text: &str, width: usize, subsequent_indent: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let words: Vec<&str> = text.split_whitespace().collect();

    if words.is_empty() {
        return lines;
    }

    let mut current_line = String::new();
    let mut first_line = true;

    for word in words {
        let space_needed = if current_line.is_empty() { 0 } else { 1 };
        let indent = if first_line { 0 } else { subsequent_indent };

        if current_line.len() + space_needed + word.len() + indent > width
            && !current_line.is_empty()
        {
            // Line would be too long, start a new line
            lines.push(current_line);
            current_line = String::new();
            first_line = false;
        }

        if !current_line.is_empty() {
            current_line.push(' ');
        }

        current_line.push_str(word);
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

fn print_wrapped_chat_line(line: &str) {
    let terminal_width = get_terminal_width();
    let base_indent = 2; // Initial 2-space indent

    // Split on existing line breaks first
    for line_part in line.lines() {
        if let Some(colon_pos) = line_part.find(": ") {
            // This is a speaker line (e.g., "Sorcerer: message" or "AppName: message")
            let speaker_prefix = &line_part[..colon_pos + 2]; // Include ": "
            let message_content = &line_part[colon_pos + 2..];

            let speaker_with_indent = format!("{}{}", " ".repeat(base_indent), speaker_prefix);
            let content_indent = speaker_with_indent.len();

            // Wrap the message content
            if message_content.is_empty() {
                println!("{speaker_with_indent}");
            } else {
                let available_width = terminal_width.saturating_sub(base_indent);
                let wrapped_lines = wrap_text(message_content, available_width, content_indent);

                for (i, wrapped_line) in wrapped_lines.iter().enumerate() {
                    if i == 0 {
                        println!("{speaker_with_indent}{wrapped_line}");
                    } else {
                        println!("{}{wrapped_line}", " ".repeat(content_indent));
                    }
                }
            }
        } else {
            // No speaker prefix, just indent and wrap
            let available_width = terminal_width.saturating_sub(base_indent);
            let wrapped_lines = wrap_text(line_part, available_width, base_indent);

            for wrapped_line in wrapped_lines {
                println!("{}{wrapped_line}", " ".repeat(base_indent));
            }
        }
    }
}

fn format_chat_line_for_pager(line: &str) -> Vec<String> {
    let mut result = Vec::new();
    let terminal_width = get_terminal_width();
    let base_indent = 2; // Initial 2-space indent

    // Split on existing line breaks first
    for line_part in line.lines() {
        if let Some(colon_pos) = line_part.find(": ") {
            // This is a speaker line (e.g., "Sorcerer: message" or "AppName: message")
            let speaker_prefix = &line_part[..colon_pos + 2]; // Include ": "
            let message_content = &line_part[colon_pos + 2..];

            let speaker_with_indent = format!("{}{}", " ".repeat(base_indent), speaker_prefix);
            let content_indent = speaker_with_indent.len();

            // Wrap the message content
            if message_content.is_empty() {
                result.push(speaker_with_indent);
            } else {
                let available_width = terminal_width.saturating_sub(base_indent);
                let wrapped_lines = wrap_text(message_content, available_width, content_indent);

                for (i, wrapped_line) in wrapped_lines.iter().enumerate() {
                    if i == 0 {
                        result.push(format!("{speaker_with_indent}{wrapped_line}"));
                    } else {
                        result.push(format!("{}{wrapped_line}", " ".repeat(content_indent)));
                    }
                }
            }
        } else {
            // No speaker prefix, just indent and wrap
            let available_width = terminal_width.saturating_sub(base_indent);
            let wrapped_lines = wrap_text(line_part, available_width, base_indent);

            for wrapped_line in wrapped_lines {
                result.push(format!("{}{wrapped_line}", " ".repeat(base_indent)));
            }
        }
    }

    result
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
