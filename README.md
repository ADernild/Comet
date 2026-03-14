# Comet 🌠

A flexible, schema-driven CLI tool for creating structured git commit messages.

## Why Comet?

Stop manually formatting git commit messages. Define your commit schema once, and Comet handles the rest with interactive prompts, validation, and consistent formatting. Whether you follow Conventional Commits or have your own commit message standards, Comet makes it easy to enforce and maintain them.

## Demo

<p align="center">
  <img src="demo.gif" alt="Comet Demo" />
</p>

## Features

- 🎯 **Interactive prompts** - Step-by-step guidance for creating git commits
- ⚙️ **Schema-driven** - Define your own git commit message format via TOML
- 📝 **Conventional commits** - Ships with Conventional Commits preset
- ✅ **Built-in validation** - Enforce message length, patterns, and required fields
- ✍️ **Git integration** - Uses `git commit` under the hood, respecting all your existing Git configuration
- 🔍 **Git-aware** - Detects repository and shows staged files

## Installation

### Via Cargo (Recommended)

```sh
cargo install git-cmt
```

### Via Installation Script

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/adernild/comet/releases/download/v0.1.0/git-cmt-installer.sh | sh

```

### Verify Installation

The binary is named `git-cmt`, which allows you to invoke it as a Git subcommand:

```sh
git cmt        # Works because the binary is git-cmt
git-cmt        # Also works if called directly
```

Make sure the binary is in your `PATH` for Git to find it.

## Quick Start

```sh
# Initialize configuration in your repository
git cmt init

# Create a commit (interactive mode)
git cmt

# Get help
git cmt -h
```

> [!NOTE]
> Use `-h` for help, not `--help`. Git intercepts `--help` to look for man pages.

## Usage

### Create a commit

```sh
git cmt
```

This will:

1. Detect and show your staged files
2. Prompt for commit details based on your schema (type, scope, description, etc.)
3. Preview the final git commit message
4. Create the commit using `git commit` (respects your existing Git config including GPG signing, hooks, etc.)

### Initialize configuration

```sh
git cmt init
```

Prompts you to choose a template and creates a `.comet.toml` file in your git repository root. Available templates:

- **Conventional Commits** - Standard format with type, scope, description, body, and footer
- **Minimal** - Simple format with just description and optional body

You can also skip the prompt by specifying a template directly:

```sh
git cmt init --conventional   # Use Conventional Commits
git cmt init --minimal         # Use minimal template
```

Once created, customize the `.comet.toml` schema to match your team's git commit message standards.

### Configuration

Comet looks for `.comet.toml` in the following locations (in order):

1. Git repository root (`.comet.toml`)
2. User config directory (`~/.config/comet/.comet.toml`)

If no config file is found, Comet uses the Conventional Commits preset by default.

### Bypass Interactive Mode

Skip prompts by providing field values directly:

```sh
# Provide specific fields
git cmt --field type=feat --field description="add new feature"

# Use --no-prompt to commit with only provided fields
git cmt --field type=fix --field description="fix bug" --no-prompt
```

This is useful for automation or scripts while still maintaining your schema format.

### Schema Reference

#### Example Schema

Here's the default Conventional Commits schema that ships with Comet:

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

[field.validate]
min = 1
max = 20

[[field]]
id = "description"
type = "text"
prompt = "Description"
required = true
help = "Brief description of changes (1-72 characters)"

[field.validate]
min = 1
max = 72

[[field]]
id = "body"
type = "multiline"
prompt = "Body (optional)"
required = false
help = "Detailed explanation of changes"
wrap = 72

[[field]]
id = "footer"
type = "text"
prompt = "Footer (optional)"
required = false
help = "Breaking changes, issue references (e.g., 'Closes #42')"

[field.validate]
pattern = "^[a-zA-Z-]+[: #].+$"

```

### Field Types

#### `select`

Choose from a predefined list of options (dropdown).

```toml
[[field]]
id = "type"
type = "select"
prompt = "Select commit type"
required = true
options = ["feat", "fix", "docs"]
```

#### `text`

Single-line text input.

```toml
[[field]]
id = "scope"
type = "text"
prompt = "Scope (optional)"
required = false
```

#### `multiline`

```toml
[[field]]
id = "body"
type = "multiline"
prompt = "Body (optional)"
required = false
wrap = 72  # Wrap text at 72 characters
```

#### `confirm`

Yes/no confirmation that can map to custom strings in the output.

```toml
[[field]]
id = "breaking"
type = "confirm"
prompt = "Breaking change?"
required = true

[field.values]
true = "!"
false = ""
```

### Validation Rules

Add validation to enforce constraints on field values:

#### `min` and `max`

Enforce minimum and maximum length:


```toml
[field.validate]
min = 1
max = 72
```

#### `pattern`

Use regex pattern matching:

```toml
[field.validate]
pattern = "^[a-zA-Z-]+[: #].+$"
```

### Template System

The `output.template` uses placeholders like `{type}`, `{scope}`, etc. that correspond to field IDs.

**Comet automatically**:

- Substitutes placeholders with user input
- Removes empty optional fields and surrounding syntax (e.g., `({scope})` becomes empty if scope is blank)
- Cleans up extra whitespace and blank lines
- Validates required fields are present

### Custom Schemas

#### Example: Jira-style commits

```toml
[output]
template = "[{ticket}] {description}\n\n{details}"

[[field]]
id = "ticket"
type = "text"
prompt = "Jira ticket"
required = true
help = "Jira ticket number (e.g., PROJ-123)"

[field.validate]
pattern = "^[A-Z]+-[0-9]+$"

[[field]]
id = "description"
type = "text"
prompt = "Description"
required = true

[[field]]
id = "details"
type = "multiline"
prompt = "Additional details"
required = false

```

### Requirements

- **Git 2.0+** - Required for commit operations
- **Rust 1.85+** - Only required if building from source

### Contributing

Contributions are welcome! Please feel free to:

- Report bugs or request features via [GitHub Issues](https://github.com/adernild/comet/issues)
- Submit pull requests
- Share your custom schemas

## License

Licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
