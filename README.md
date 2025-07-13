# 🧙‍♂️ Sorcerer - The Magical AI Orchestrator

> *"The old sorcerer himself was not present to watch over his agent..."*

An orchestration tool for managing AI agents. Sorcerer lets you spawn, manage,
and manage agents running in isolated containers, and
orchestrate them to collaborate on collaborative tasks. An agent may be allowed
to take over orchestration for fully automated problem solving, but the
sorcerer should keep an eye on his agent, lest he relearn old lessons.

<div align="center">

![Sorcerer's Apprentice](docs/assets/flamenquines-don-chalecos.gif)

</div>

## 📜 The Legend

Tool allows you to:
- **Summon agents** - spawn a new agent in its own container
- **List your domain** - list active agents
- **Get overview** - check detailed status
- **Kill them** - stop agent processes

Each agent runs in its own isolated container, communicating through gRPC.

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

### Building the Artifacts

```bash
# Build the agent image (works with both Podman and Docker)
cd sorcerer
podman build -f agent/Containerfile -t sorcerer-agent:latest .
# OR: docker build -f agent/Containerfile -t sorcerer-agent:latest .

# Build the sorcerer CLI
cargo build --release
```

### Creating Your First Apprentice

```bash
# Create an agent named "Mickey"
./target/release/srcrr create Mickey

# Create multiple agents at once
./target/release/srcrr create Alice Bob Carol

# Check on all your agents
./target/release/srcrr list

# When done, remove Mickey
./target/release/srcrr rm Mickey

# Remove multiple agents at once
./target/release/srcrr rm Alice Bob

# Remove all agents
./target/release/srcrr rm -a
```

## 🔮 Commands of Power

### `srcrr create <name>...`
Brings forth one or more agents from the mystical realm. Each agent is bound to serve until removed. You can create multiple agents by providing multiple names.

### `srcrr list`
Reveals all agents currently in your service. A simple way to see who answers to your call.

### `srcrr ps`
Shows detailed information about each agent's state and recent activity.

### `srcrr rm <name>... | -a`
Stops and removes one or more agent containers, cleaning up all traces of their existence. You can remove multiple agents by providing multiple names, or use `-a`/`--all` to remove all agents at once.

## 🏗️ Architecture

```
┌─────────────┐         gRPC           ┌────────────────┐
│   Sorcerer  │◄──────────────────────►│   Apprentice   │
│    (CLI)    │                        │    (Agent)     │
└─────────────┘                        └────────────────┘
      │                                        
      │                                        
      ▼                                        
 Podman/Docker API
```

- **Sorcerer**: The master process that orchestrates everything
- **Apprentices**: Individual containers running gRPC servers
- **Spells Protocol**: gRPC-based communication between sorcerer and agents
- **Isolation**: Each agent operates in its own container
- **Runtime**: Supports both Podman (rootless) and Docker (with daemon)

## ⚠️ Words of Warning

Be careful not to create more helpers than you can manage. Each agent
consumes resources.

Remember to kill your agents when done - they won't clean up after
themselves!

## 🎭 In the Spirit of the Tale

This tool embodies the whimsical yet powerful nature of The Sorcerer's
Apprentice. While our agents won't flood your workshop with water, they
will faithfully execute your commands through the magic of AI.

Use this power wisely, young sorcerer!
