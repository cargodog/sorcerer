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
- **Scry upon your domain** - list active apprentices
- **Consult the grimoire** - check detailed status
- **Banish them** - stop agent processes

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

# Send a message to Mickey
./target/release/srcrr tell Mickey "What is the meaning of life?"

# Check on all your apprentices
./target/release/srcrr scry

# When done, banish Mickey back to the void
./target/release/srcrr banish Mickey
```

## 🔮 Commands of Power

### `srcrr summon <name>`
Brings forth a new apprentice from the mystical realm. Each apprentice is bound to serve until banished.

### `srcrr tell <name> "<message>"`
Sends a message to an apprentice (sends a prompt to Claude). The apprentice will channel the wisdom of the ancients to fulfill your request.

### `srcrr scry`
Reveals all apprentices currently in your service. A simple divination to see who answers to your call.

### `srcrr grimoire`
Opens the book of knowledge, revealing detailed information about each apprentice's state and recent activity.

### `srcrr banish <name>`
Sends an apprentice back to the ethereal plane, cleaning up all traces of their existence.

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

Remember to banish your apprentices when done - they won't clean up after
themselves!

## 🎭 In the Spirit of the Tale

This tool embodies the whimsical yet powerful nature of The Sorcerer's
Apprentice. While our apprentices won't flood your workshop with water, they
will faithfully execute your commands through the magic of AI.

Use this power wisely, young sorcerer!
