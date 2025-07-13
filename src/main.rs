mod config;
mod sorcerer;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "srcrr")]
#[command(about = "🧙‍♂️ The Sorcerer - Command agents to do your bidding")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create and start new agent containers
    Create {
        /// Names of the agents to create
        names: Vec<String>,
    },
    /// List all active agents
    List,
    /// Stop and remove agent containers
    Rm {
        /// Names of the agents to remove
        names: Vec<String>,
        /// Remove all agents
        #[arg(short, long)]
        all: bool,
    },
    /// Show detailed status information for all agents
    Ps {
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
        Commands::Create { names } => {
            if names.is_empty() {
                println!("❌ No agent names provided");
                return Ok(());
            }

            let total = names.len();

            // Print initial messages
            for name in &names {
                println!("🌟 Creating agent {name}...");
            }

            // Execute creations concurrently
            let tasks: Vec<_> = names
                .into_iter()
                .map(|name| {
                    let name_clone = name.clone();
                    let sorcerer = &sorcerer;
                    async move {
                        let result = sorcerer.create_agent(&name).await;
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
                        println!("✨ Agent {name} has answered your call!");
                        successes += 1;
                    }
                    Err(e) => {
                        error!("Failed to create agent {}: {}", name, e);
                        println!("💀 Failed to create {name}");
                    }
                }
            }

            if total > 1 {
                println!("\n📊 Summary: {successes}/{total} agents created successfully");
            }
        }
        Commands::List => {
            println!("📋 Listing agents...");
            println!();
            let agents = sorcerer.list_agents().await?;
            if agents.is_empty() {
                println!("The realm is empty - no agents found.");
            } else {
                for agent in agents {
                    println!("🧙 {agent}");
                }
            }
        }
        Commands::Rm { names, all } => {
            let agents_to_remove = if all {
                let all_agents = sorcerer.list_agents().await?;
                if all_agents.is_empty() {
                    println!("📭 No agents to remove");
                    return Ok(());
                }
                println!("🗑️  Removing all {} agents...", all_agents.len());
                all_agents
            } else {
                if names.is_empty() {
                    println!("❌ No agent names provided (use -a for all)");
                    return Ok(());
                }
                names
            };

            let total = agents_to_remove.len();
            let mut successes = 0;

            // Print initial messages
            for name in &agents_to_remove {
                println!("💀 Removing agent {name}...");
            }

            // Execute removals concurrently
            let tasks: Vec<_> = agents_to_remove
                .into_iter()
                .map(|name| {
                    let name_clone = name.clone();
                    let sorcerer = &sorcerer;
                    async move {
                        let result = sorcerer.remove_agent(&name).await;
                        (name_clone, result)
                    }
                })
                .collect();

            let results = futures::future::join_all(tasks).await;

            // Process results
            for (name, result) in results {
                match result {
                    Ok(_) => {
                        println!("⚰️  Agent {name} has been removed!");
                        successes += 1;
                    }
                    Err(e) => {
                        error!("Failed to remove agent {}: {}", name, e);
                        println!("⚠️  Failed to remove {name}");
                    }
                }
            }

            if total > 1 {
                println!("\n📊 Summary: {successes}/{total} agents removed successfully");
            }
        }
        Commands::Ps { lines } => {
            println!("📊 Overview of agents...");
            let statuses = sorcerer.get_all_status().await?;
            if statuses.is_empty() {
                println!("No agents found.");
            } else {
                let mut first = true;
                for (name, status) in statuses {
                    if !first {
                        println!(); // Add spacing between apprentices
                    }
                    first = false;

                    // Calculate box width based on agent name length
                    let min_width = 45;
                    let name_header = format!(" Agent: {name} ");
                    let box_width = min_width.max(name_header.len() + 2);

                    // Draw agent info box
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
                username if username.contains("agent-") => {
                    // Mild green for agents
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
