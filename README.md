# SSHING

A modern, terminal-based SSH connection manager with an intuitive TUI interface.

---

## Table of Contents

- [What is sshing?](#what-is-sshing)
- [Features](#features)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage Guide](#usage-guide)
  - [Navigation](#navigation)
  - [Managing Hosts](#managing-hosts)
  - [SSH Flags & Shell Selection](#ssh-flags--shell-selection)
  - [Tags & Organization](#tags--organization)
  - [Search & Filter](#search--filter)
- [Keyboard Shortcuts](#keyboard-shortcuts)
- [Configuration](#configuration)
- [Contributing](#contributing)
- [License](#license)

---

## What is sshing?

**sshing** is a powerful SSH connection manager that simplifies the way you manage and connect to remote servers. Instead of manually editing `~/.ssh/config` or remembering complex SSH commands, sshing provides a beautiful terminal interface where you can:

- Browse all your SSH hosts in a visual table
- Add, edit, and delete host configurations with validation
- Organize hosts with tags (prod, staging, dev, etc.)
- Search and filter hosts in real-time
- Connect to any server with a single keystroke
- Customize SSH flags and shell per connection

All your configuration is stored in standard SSH config format, so sshing works seamlessly with existing SSH tools and workflows.

[↑ Back to Top](#table-of-contents)

---

## Features

### Core Features
- **Instant SSH Connections** - Press Space to connect to any host
- **Visual Host Management** - Full CRUD operations with an intuitive form editor
- **Vim-Style Navigation** - Navigate with `j/k`, jump with `g/G`, page with `Ctrl+d/u`
- **Real-Time Search** - Filter hosts as you type with live overlay
-  **Tag System** - Organize hosts with a global tag pool (prod, staging, dev, etc.)
- 🔑 **SSH Key Selector** - Visual picker to select multiple keys per host
- ⚙️ **SSH Flags & Shell Selection** - Customize connection behavior per host
- **Jump Host Support** - Configure ProxyJump for bastion hosts
- **Flexible Sorting** - Sort by name, hostname, last used, or user
-  **Usage Tracking** - Automatically track when you last connected to each host

### Technical Features
- **SSH Config Integration** - Reads from and writes to `~/.ssh/config`
- **Extended Metadata** - Stores notes, tags, flags, and timestamps in `~/.ssh/sshing.json`
- **Proper Terminal Handling** - Cleanly transitions between TUI and SSH sessions
- **Input Validation** - Prevents duplicate hosts and invalid configurations
- **Confirmation Dialogs** - Prevents accidental deletions

[↑ Back to Top](#table-of-contents)

---

## Quick Start

### Installation

**From Source:**
```bash
git clone https://github.com/joshjetson/sshing.git
cd sshing
cargo build --release
sudo cp target/release/sshing /usr/local/bin/
```

**Using Cargo:**
```bash
cargo install sshing
```

### First Use

1. **Launch sshing:**
   ```bash
   sshing
   ```

2. **Add your first host:**
   - Press `n` to create a new host
   - Fill in the form (use `j/k` or `Tab` to navigate):
     - **Host (alias)**: `my-server`
     - **Hostname (IP)**: `192.168.1.10`
     - **User**: `ubuntu`
     - **Port**: `22` (or leave empty for default)
   - Press `Ctrl+S` to save

3. **Connect to your host:**
   - Select the host with `j/k` or arrow keys
   - Press `Space` or `Enter` to connect

That's it! You're now SSH'd into your server.

[↑ Back to Top](#table-of-contents)

---

## Installation

### Prerequisites
- Rust 1.70 or later
- Cargo package manager
- SSH client installed on your system
- Need rsync to use the rsync features

### Option 1: Install from Source

```bash
# Clone the repository
git clone https://github.com/joshjetson/sshing.git
cd sshing

# Build the release binary
cargo build --release

# Copy to your PATH
sudo cp target/release/sshing /usr/local/bin/

# Or install to user directory
cp target/release/sshing ~/.local/bin/
```

### Option 2: Install with Cargo

```bash
cargo install sshing
```

### Option 3: Download Pre-built Binary

Download the latest release from [GitHub Releases](https://github.com/joshjetson/sshing/releases) and add it to your PATH.

### Verify Installation

```bash
sshing --version
```

[↑ Back to Top](#table-of-contents)

---

## Usage Guide

### Navigation

sshing uses intuitive Vim-style keybindings:

- **Move Down:** `j` or `↓`
- **Move Up:** `k` or `↑`
- **Jump to Top:** `g`
- **Jump to Bottom:** `G`
- **Page Down:** `Ctrl+d`
- **Page Up:** `Ctrl+u`

The currently selected host is highlighted with a cyan bar.

### Managing Hosts

#### Creating a New Host

1. Press `n` from the main table view
2. Fill in the form fields:
   - **Host (alias)** - Unique name for this connection
   - **Hostname (IP)** - Server IP address or domain
   - **User** - SSH username (optional)
   - **Port** - SSH port (default: 22)
   - **SSH Keys** - Press `Enter` to select identity files
   - **Jump Host** - ProxyJump configuration (optional)
   - **SSH Flags** - Press `Enter` to select flags like `-t`, `-A`, etc.
   - **Shell** - Press `Enter` to select shell (bash, zsh, fish, etc.)
   - **Tags** - Press `Enter` to assign tags
   - **Note** - Personal notes about this server
3. Navigate fields with `j/k`, `↑/↓`, or `Tab`
4. Press `Ctrl+S` to save

#### Editing a Host

1. Select the host with `j/k`
2. Press `e` to edit
3. Modify fields (same navigation as creating)
4. Press `Ctrl+S` to save or `Esc` to cancel

#### Deleting a Host

1. Select the host with `j/k`
2. Press `d` to delete
3. Confirm with `y` or `Enter` (or cancel with `n` or `Esc`)

### SSH Flags & Shell Selection

#### SSH Flags

SSH flags customize how the connection is established. Common use cases:

- **`-t`** - Force pseudo-terminal (needed for interactive shells like `zsh`)
- **`-A`** - Enable SSH agent forwarding (use your local SSH keys on remote)
- **`-X` / `-Y`** - Enable X11 forwarding (run GUI apps remotely)
- **`-C`** - Enable compression (faster on slow connections)
- **`-v` / `-vv` / `-vvv`** - Verbose mode for debugging

**To add flags to a host:**
1. Edit the host (`e`)
2. Navigate to the **SSH Flags** field
3. Press `Enter` to open the flag selector
4. Use `j/k` to navigate, `Space` or `Enter` to toggle flags
5. Press `Esc` to return to the editor
6. Press `Ctrl+S` to save

#### Shell Selection

Choose which shell to execute after connecting. Useful when:
- Your server's default shell is different from your preference
- You want to always start in `zsh` on certain servers
- You need a specific shell for scripting

**To set a shell:**
1. Edit the host (`e`)
2. Navigate to the **Shell** field
3. Press `Enter` to open the shell selector
4. Use `j/k` to navigate, `Space` or `Enter` to select
5. Press `Esc` to return to the editor
6. Press `Ctrl+S` to save

**Available shells:** bash, zsh, fish, sh, ksh, tcsh, dash

### Tags & Organization

Tags help you organize hosts by environment, role, or any category you choose.

#### Global Tag Pool

sshing maintains a **global tag pool** that's shared across all hosts. When you create a tag, it's saved to the pool and can be assigned to any host.

#### Creating Tags

1. Edit any host (`e`)
2. Navigate to the **Tags** field
3. Press `Enter` to open the tag editor
4. Press `a` or `n` to create a new tag
5. Type the tag name (e.g., `production`, `staging`, `web`, `database`)
6. Press `Enter` to add it to the global pool
7. The tag is now available but **not automatically assigned** to the current host

#### Assigning Tags to Hosts

1. In the tag editor, use `j/k` to navigate the tag list
2. Press `Space` or `Enter` to toggle tag assignment
3. Selected tags show a `[✓]` checkbox
4. Press `Esc` to return to the editor
5. Press `Ctrl+S` to save

#### Filtering by Tags

1. Press `t` from the main table view
2. Select tags to filter by
3. Press `Enter` to apply the filter
4. Press `Esc` to clear all filters

### Search & Filter

#### Real-Time Search

1. Press `/` to enter search mode
2. Start typing - the table filters as you type
3. Press `Enter` to apply the search
4. Press `Esc` to clear the search

Search matches against:
- Host alias
- Hostname/IP
- User
- Tags
- Notes

#### Sorting

Press `s` to cycle through sort options:
- **Name** - Sort by host alias alphabetically
- **Hostname** - Sort by IP/hostname
- **Last Used** - Most recently used first
- **User** - Sort by username

[↑ Back to Top](#table-of-contents)

---

## Keyboard Shortcuts

### Main Table View

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `g` | Jump to first host |
| `G` | Jump to last host |
| `Ctrl+d` | Page down |
| `Ctrl+u` | Page up |
| `Space` / `Enter` | Connect to selected host |
| `n` | Create new host |
| `e` | Edit selected host |
| `d` | Delete selected host |
| `/` | Search hosts |
| `t` | Filter by tags |
| `s` | Cycle sort order |
| `Esc` | Clear filters/search |
| `?` | Show help |
| `q` | Quit application |

### Edit Host Form

| Key | Action |
|-----|--------|
| `j` / `k` / `↑` / `↓` | Navigate fields (navigation mode) |
| `Tab` / `Shift+Tab` | Navigate fields |
| `Enter` | Activate editing mode / Open special editors |
| `Esc` | Exit editing mode / Cancel |
| `Ctrl+S` | Save host |

**When editing a field:**
- Type to input text
- `Backspace` to delete characters
- `Enter` to save field
- `Esc` to cancel changes to current field

### SSH Key Selection

| Key | Action |
|-----|--------|
| `j` / `k` / `↑` / `↓` | Navigate keys |
| `Space` / `Enter` | Toggle key selection |
| `Esc` | Return to editor |

### SSH Flags Selection

| Key | Action |
|-----|--------|
| `j` / `k` / `↑` / `↓` | Navigate flags |
| `Space` / `Enter` | Toggle flag selection |
| `Esc` | Return to editor |

### Shell Selection

| Key | Action |
|-----|--------|
| `j` / `k` / `↑` / `↓` | Navigate shells |
| `Space` / `Enter` | Select/deselect shell |
| `Esc` | Return to editor |

### Tag Editor

| Key | Action |
|-----|--------|
| `j` / `k` / `↑` / `↓` | Navigate tags (selection mode) |
| `Space` / `Enter` | Toggle tag assignment |
| `a` / `n` / `i` | Create new tag (enter input mode) |
| `Esc` | Return to editor |

**When creating a new tag:**
- Type the tag name
- `Enter` to add to global pool
- `Esc` to cancel

### Search Mode

| Key | Action |
|-----|--------|
| Type | Filter hosts in real-time |
| `Backspace` | Delete character |
| `Enter` | Apply search |
| `Esc` | Cancel search |

### Delete Confirmation

| Key | Action |
|-----|--------|
| `y` / `Y` / `Enter` | Confirm deletion |
| `n` / `N` / `Esc` | Cancel |

[↑ Back to Top](#table-of-contents)

---

## Configuration

### SSH Config File

sshing stores standard SSH configuration in `~/.ssh/config`:

```
Host my-server
    HostName 192.168.1.10
    User ubuntu
    Port 22
    IdentityFile ~/.ssh/id_rsa
    ProxyJump bastion
```

This file is compatible with the standard `ssh` command and other SSH tools.

### Metadata File

Extended metadata (notes, tags, SSH flags, shell, last used) is stored in `~/.ssh/sshing.json`:

```json
{
  "version": "1.0",
  "global_tags": [
    "prod",
    "staging",
    "dev",
    "web",
    "database"
  ],
  "hosts": {
    "my-server": {
      "note": "Main production web server",
      "tags": ["prod", "web"],
      "ssh_flags": ["-t", "-A"],
      "shell": "zsh",
      "last_used": "2025-12-31T10:30:00Z"
    }
  }
}
```

### File Locations

- **SSH Config:** `~/.ssh/config`
- **Metadata:** `~/.ssh/sshing.json`

### Backwards Compatibility

If you already have an existing `~/.ssh/config` file, sshing will:
1. Read all existing Host entries
2. Preserve any configuration you've manually added
3. Allow you to edit hosts through the TUI
4. Only modify hosts that you edit through sshing

[↑ Back to Top](#table-of-contents)

---

## Contributing

Contributions are welcome! Here's how you can help:

1. **Fork the repository**
2. **Create a feature branch:** `git checkout -b feature/amazing-feature`
3. **Commit your changes:** `git commit -m 'Add amazing feature'`
4. **Push to the branch:** `git push origin feature/amazing-feature`
5. **Open a Pull Request**

### Development Setup

```bash
# Clone your fork
git clone https://github.com/joshjetson/sshing.git
cd sshing

# Build and run
cargo build
cargo run

# Run tests
cargo test

# Check code quality
cargo clippy
cargo fmt
```

### Reporting Issues

Found a bug? Have a feature request? Please [open an issue](https://github.com/joshjetson/sshing/issues) with:
- A clear description of the problem or suggestion
- Steps to reproduce (for bugs)
- Your environment (OS, Rust version, terminal emulator)

[↑ Back to Top](#table-of-contents)

---

## License

MIT License - see [LICENSE](LICENSE) file for details.

---

## Acknowledgments

Built with:
- **[Rust](https://www.rust-lang.org/)** - Systems programming language
- **[Ratatui](https://github.com/ratatui-org/ratatui)** - Terminal UI framework
- **[Crossterm](https://github.com/crossterm-rs/crossterm)** - Terminal manipulation library
- **[Serde](https://serde.rs/)** - Serialization framework

[↑ Back to Top](#table-of-contents)
