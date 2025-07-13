# Contributing to Sorcerer

## Getting Started

1. **Prerequisites**
   - Rust 1.75+ 
   - Podman or Docker

2. **Setup**
   ```bash
   git clone <repo-url>
   cd sorcerer
   ./install-hooks.sh  # Install git hooks
   ./build.sh          # Build project and container
   ```

## Development Workflow

### Code Standards
- **Formatting**: `cargo fmt` (enforced by pre-commit hook)
- **Linting**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Building**: `cargo build` must pass
- **Testing**: `cargo test` for all changes

### Git Workflow
- Make small, atomic commits
- All git hooks run automatically (never use `--no-verify`)
- Pre-commit hook checks formatting, linting, builds, and security
- Commit messages should be clear and concise

### Testing
Run tests with:
```bash
cargo test                    # Unit and integration tests
cargo test --test <name>      # Specific test file
```

Test files are in `/tests/` directory covering:
- Unit tests
- Integration tests  
- Container operations
- Error handling
- Apprentice functionality

### Security
- No hardcoded secrets/keys in code
- Minimal debug prints in production code
- Pre-commit hook performs basic security checks

## Project Structure
```
sorcerer/
├── src/           # Core sorcerer CLI
├── agent/    # Agent container code
├── tests/         # Test suite
├── proto/         # gRPC protocol definitions
└── hooks/         # Git hooks
```

## Submitting Changes

1. Ensure all hooks pass: `git commit` (hooks run automatically)
2. Verify tests pass: `cargo test`
3. Build containers: `./build.sh`
4. Test with real agents if applicable

## Container Development

Apprentice container changes require rebuilding:
```bash
podman build -f agent/Containerfile -t sorcerer-agent:latest .
```