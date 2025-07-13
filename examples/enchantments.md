# 📖 Book of Enchantments

A collection of example spells to cast through your agents.

## Basic Incantations

```bash
# Simple knowledge request
srcrr spell Mickey "Explain quantum computing in simple terms"

# Code generation
srcrr spell Mickey "Write a Python function to calculate Fibonacci numbers"

# Creative writing
srcrr spell Mickey "Write a haiku about Docker containers"
```

## Advanced Sorcery

```bash
# Analysis and reasoning
srcrr spell Merlin "Analyze the pros and cons of microservices architecture"

# Problem solving
srcrr spell Gandalf "How would you optimize a slow SQL query that joins 5 tables?"

# Educational content
srcrr spell Dumbledore "Create a lesson plan for teaching recursion to beginners"
```

## Multiple Apprentices

```bash
# Create a council of agents with one command
srcrr create Alice Bob Carol

# Have them work on different tasks
srcrr spell Alice "Design a REST API for a todo application"
srcrr spell Bob "Write unit tests for a shopping cart class"
srcrr spell Carol "Create a deployment checklist for a web application"

# Check their progress
srcrr ps

# Dismiss the council with one command
srcrr rm Alice Bob Carol

# Or dismiss all agents at once
# srcrr rm -a
```

## Debugging Spells

```bash
# When something goes wrong
srcrr spell Helper "Debug this error: NullPointerException at line 42"

# Code review
srcrr spell Reviewer "Review this code for security vulnerabilities: [paste code]"

# Performance optimization
srcrr spell Optimizer "How can I make this algorithm run faster: [describe algorithm]"
```

## Remember

Each spell costs mana (API tokens), so use your power wisely!