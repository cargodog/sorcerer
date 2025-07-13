# 🧙‍♂️ Sorcerer - The Magical AI Orchestrator

> *"The old sorcerer himself was not present to watch over his apprentice..."*

An orchestration tool for managing AI agents. Sorcerer lets you spawn, manage,
and communicate with Claude AI agents running in isolated containers, and
orchestrate them to collaborate on collaborative tasks. An agent may be allowed
to take over orchestration for fully automated problem solving, but the
sorcerer should keep an eye on his apprentice, lest he relearn old lessons.

<div align="center">

![Sorcerer's Apprentice](docs/assets/flamenquines-don-chalecos.gif)

</div>

## 📜 The Legend

Tool allows you to:
- **Summon apprentices** - spawn a new agent in its own container
- **Command them** - send chat prompts
- **List your domain** - list active apprentices
- **Get overview** - check detailed status
- **Kill them** - stop agent processes

Each apprentice runs in its own isolated container, communicating through gRPC.

## 🌟 Quick Start

### Prerequisites

1. **Container Runtime** - Either Podman (recommended) or Docker
   
   **Option A: Podman (Recommended - Rootless & Daemonless)**
   ```bash
   # On Arch Linux
   sudo pacman -S podman
   
   # Start user podman socket
   systemctl --user start podman.socket
   systemctl --user enable podman.socket
   ```
   
   **Option B: Docker**
   ```bash
   # On Arch Linux  
   sudo pacman -S docker
   
   # Start Docker daemon
   sudo systemctl start docker
   sudo systemctl enable docker
   
   # Add your user to docker group
   sudo usermod -aG docker $USER
   # Then logout and login again
   ```

2. **Rust** toolchain (1.75 or later)
3. **Anthropic API Key** for Claude

### Building the Artifacts

```bash
# Build the apprentice image (works with both Podman and Docker)
cd sorcerer
podman build -f apprentice/Containerfile -t sorcerer-apprentice:latest .
# OR: docker build -f apprentice/Containerfile -t sorcerer-apprentice:latest .

# Build the sorcerer CLI
cargo build --release
```

### Summoning Your First Apprentice

```bash
# Set your Claude API key
export ANTHROPIC_API_KEY="your-key-here"

# Summon an apprentice named "Mickey"
./target/release/srcrr summon Mickey

# Summon multiple apprentices at once
./target/release/srcrr summon Alice Bob Carol

# Send a message to Mickey
./target/release/srcrr tell Mickey "What is the meaning of life?"

# Check on all your apprentices
./target/release/srcrr ls

# When done, remove Mickey
./target/release/srcrr rm Mickey

# Remove multiple apprentices at once
./target/release/srcrr rm Alice Bob

# Remove all apprentices
./target/release/srcrr rm -a
```

## 🤖 Autonomous Apprentices

By default, apprentices can perform autonomous tasks! They can execute commands, manipulate files, and complete complex workflows independently.

```bash
# Summon an autonomous apprentice (default behavior)
./target/release/srcrr summon CodeWizard

# Give it a complex task
./target/release/srcrr tell CodeWizard "Find all TODO comments in the codebase and create a summary report"

# The apprentice will autonomously:
# - Search through files
# - Analyze findings  
# - Create a report
# - Present results
```

Apprentices can:
- 📄 Read, write, and edit files
- 🔍 Search codebases with ripgrep
- 💻 Execute shell commands
- 🌐 Fetch web content
- 📊 Parse structured data
- 🧠 Plan and track multi-step tasks

For simple chat without autonomous capabilities, use `--no-system-prompt`.

See [docs/agent-mode.md](docs/agent-mode.md) for detailed documentation.

## 🔮 Commands of Power

### `srcrr summon <name>... [--no-system-prompt]`
Brings forth one or more autonomous apprentices from the mystical realm. Each apprentice is bound to serve until removed. You can summon multiple apprentices by providing multiple names.

Use `--no-system-prompt` to summon non-autonomous apprentices for simple chat only.

### `srcrr tell <name> "<message>"`
Sends a message to an apprentice (sends a prompt to Claude). The apprentice will channel the wisdom of the ancients to fulfill your request.

### `srcrr ls`
Reveals all apprentices currently in your service. A simple way to see who answers to your call.

### `srcrr ps`
Shows detailed information about each apprentice's state and recent activity.

### `srcrr rm <name>... | -a`
Stops and removes one or more apprentice containers, cleaning up all traces of their existence. You can remove multiple apprentices by providing multiple names, or use `-a`/`--all` to remove all apprentices at once.

## 🏗️ Architecture

```
┌─────────────┐         gRPC           ┌────────────────┐
│   Sorcerer  │◄──────────────────────►│   Apprentice   │
│    (CLI)    │                        │    (Agent)     │
└─────────────┘                        └────────────────┘
      │                                         │
      │                                         │
      ▼                                         ▼
 Podman/Docker API                         Claude API
```

- **Sorcerer**: The master process that orchestrates everything
- **Apprentices**: Individual containers running gRPC servers
- **Spells Protocol**: gRPC-based communication for reliable incantations
- **Isolation**: Each apprentice operates in its own container
- **Runtime**: Supports both Podman (rootless) and Docker (with daemon)

## ⚠️ Words of Warning

Be careful not to summon more helpers than you can manage. Each apprentice
consumes resources and makes API calls to Claude.

Remember to kill your apprentices when done - they won't clean up after
themselves!

## 🎭 In the Spirit of the Tale

This tool embodies the whimsical yet powerful nature of The Sorcerer's
Apprentice. While our apprentices won't flood your workshop with water, they
will faithfully execute your commands through the magic of AI.

Use this power wisely, young sorcerer!
