# Autonomous Apprentices

This document describes the autonomous capabilities of apprentices, which is the default mode of operation.

## Overview

By default, apprentices can:
- Read, write, and edit files
- Execute shell commands
- Search through code and files
- Fetch web content
- Plan and track tasks
- Store and recall information

## Autonomous Mode (Default)

Apprentices are autonomous by default. For simple chat without autonomous capabilities, you can disable this:

```bash
# Default: Autonomous apprentice
sorcerer summon CodeWizard

# Simple chat only (no system prompt)
sorcerer summon ChatBot --no-system-prompt
```

You can also customize the system prompt using environment variables:

```bash
# Use a custom system prompt
docker run -e SYSTEM_PROMPT_PATH=/path/to/prompt.md -e ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY sorcerer-apprentice
```

## Available Commands

### File Operations
- `Read`: Read file contents
- `Write`: Create or overwrite files  
- `Edit`: Find and replace text in files
- `Delete`: Remove files

### System Operations
- `Exec`: Execute shell commands
- `List`: List directory contents with optional pattern matching
- `Search`: Search for patterns in files (uses ripgrep)

### Planning & Context
- `Think`: Log reasoning steps
- `Plan`: Create a task plan
- `UpdatePlan`: Update task status
- `Remember`: Store information for later use
- `Recall`: Retrieve previously stored information

### External Resources
- `WebFetch`: Fetch content from URLs
- `Parse`: Parse structured data (JSON, YAML, TOML)

### Reporting
- `Status`: Log status messages
- `Report`: Generate formatted reports

## Example Usage

```bash
# Summon an autonomous apprentice (default)
sorcerer summon agent

# Ask it to perform a task
sorcerer tell agent "Find all TODO comments in Python files and create a summary report"

# The apprentice will:
# 1. Use Think to plan the approach
# 2. Use Search to find TODO comments
# 3. Use Write to create a summary file
# 4. Use Report to present results
```

## Security Considerations

Autonomous apprentices can:
- Read and write files on the container filesystem
- Execute commands within the container
- Make web requests

The apprentice runs in an isolated container environment, limiting potential impact. However, use caution when:
- Mounting host directories into the container
- Running with elevated privileges
- Allowing network access to sensitive resources

## Custom System Prompts

Create a custom system prompt to specialize your apprentice's behavior:

```markdown
# Research Assistant Prompt

You are a research assistant specializing in code analysis. When given a task:

1. Always start with Think to plan your approach
2. Use Search extensively to understand the codebase
3. Create detailed reports with your findings
4. Focus on code quality, patterns, and potential improvements

Respond only with command JSON, no conversational text.
```

## Command Response Format

Apprentices in agent mode must respond with valid JSON:

```json
{
  "commands": [
    {"cmd": "Think", "reasoning": "I need to search for TODO comments in Python files"},
    {"cmd": "Search", "pattern": "TODO", "file_type": "py"},
    {"cmd": "Status", "message": "Found TODOs, creating report", "level": "info"}
  ]
}
```

## Troubleshooting

### Commands not executing
- Check if apprentice was summoned with `--no-system-prompt` flag (disables autonomy)
- Check apprentice logs for parsing errors
- Verify JSON response format is correct

### File operations failing
- Check file paths are absolute, not relative
- Ensure apprentice has necessary permissions
- Verify the file exists (for read/edit operations)

### Search not working
- The Search command requires `ripgrep` (rg) to be installed in the container
- Use the `pattern` parameter for regex searches
- Use `file_type` to filter by language