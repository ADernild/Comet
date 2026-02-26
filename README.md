# Comet 🌠

A flexible, config-driven CLI tool for creating structured commit messages.

## Features

- 🎯 **Interactive prompts** - Step-by-step guidance for creating commits
- ⚙️ **Configurable** - Define your own commit message format via TOML
- ✍️ **GPG signing** - Automatic commit signing support
- 📝 **Conventional commits** - Ships with conventional commits preset
- 🔍 **Git-aware** - Works from any directory in your repository

## Installation

```sh
cargo install --path .
```

Or add the binary to your `PATH`.

## Usage

### Create a commit

```sh
git cmt
```

This will:

1. Show staged files
2. Prompt for commit details (type, scope, description, etc.)
3. Preview the commit message
4. Create the commit (with GPG signing if enabled)

### Initialize configuration

```sh
git cmt init
```

Creates a .comet.toml in your repository root with the conventional commits preset.

### Configuration

Comet looks for `.comet.toml` in your git repository root. If not found, it uses the conventional commits preset.

```toml
[output]
template = """
{type}({scope}): {description}

{body}

{footer}"""

[[field]]
id = "type"
type = "select"
prompt = "Select commit type"
required = true
help = "Type of change you're committing"
options = [
    "feat",
    "fix",
    "docs",
    "style",
    "refactor",
    "test",
    "chore",
]

[[field]]
id = "scope"
type = "text"
prompt = "Scope (optional)"
required = false
help = "Component affected (e.g., api, auth, ui)"
options = []

[field.validate]
min = 1
max = 20

[[field]]
id = "description"
type = "text"
prompt = "Description"
required = true
help = "Brief description of changes (1-72 characters)"
options = []

[field.validate]
min = 1
max = 72

[[field]]
id = "body"
type = "multiline"
prompt = "Body (optional)"
required = false
help = "Detailed explanation of changes"
options = []
wrap = 72

[[field]]
id = "footer"
type = "text"
prompt = "Footer (optional)"
required = false
help = "Breaking changes, issue references (e.g., 'Closes #42'"
options = []

[field.validate]
pattern = "^[a-zA-Z-]+[: #].+$"

```

### Field Types

- **select** - Choose from predefined options
- **text** - single-line text input
- **multiline** - Multi-line text input (with optional wrapping)
- **confirm** - Yes/no confirmation

### Validation

Fields support validation rules:

- min - Minimum length
- max - Maximum length
- pattern - Regex pattern matching

### Requirements

- Rust 1.85+ 
- Git 2.0+
- GPG (optional, for signed commits)
