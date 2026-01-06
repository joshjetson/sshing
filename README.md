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
- [Docker Container Management](#docker-container-management)
  - [Entering Docker Mode](#entering-docker-mode)
  - [Container List View](#container-list-view)
  - [Container Actions](#container-actions)
  - [Container Inspection Tools](#container-inspection-tools)
  - [Deployment Scripts](#deployment-scripts)
- [Rsync File Synchronization](#rsync-file-synchronization)
  - [Entering Rsync Mode](#entering-rsync-mode)
  - [Using the File Browser](#using-the-file-browser)
  - [Executing Rsync](#executing-rsync)
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
- **Manage Docker containers** on remote servers with full control over logs, stats, and deployment scripts
- **Sync files with rsync** using an interactive file browser

All your configuration is stored in standard SSH config format, so sshing works seamlessly with existing SSH tools and workflows.

[↑ Back to Top](#table-of-contents)

---

## Features

### Core Features
- **Instant SSH Connections** - Press Space to connect to any host
- **Visual Host Management** - Full CRUD operations with an intuitive form editor
- **Vim-Style Navigation** - Navigate with `j/k`, jump with `g/G`, page with `Ctrl+d/u`
- **Real-Time Search** - Filter hosts as you type with live overlay
- **Tag System** - Organize hosts with a global tag pool (prod, staging, dev, etc.)
- **SSH Key Selector** - Visual picker to select multiple keys per host
- **SSH Flags & Shell Selection** - Customize connection behavior per host
- **Jump Host Support** - Configure ProxyJump for bastion hosts
- **Flexible Sorting** - Sort by name, hostname, last used, user, or tags
- **Usage Tracking** - Automatically track when you last connected to each host

### Docker Container Management
- **Container Overview** - View all containers with status, image, ports at a glance
- **Container Actions** - Start, stop, restart, pull, remove, and purge containers
- **Log Viewer** - View container logs with follow mode and adjustable line counts
- **Stats Monitor** - Real-time CPU and memory usage visualization
- **Process Viewer** - See running processes inside containers (docker top)
- **Container Inspect** - Deep dive into container configuration, ports, volumes, networks
- **Environment Inspector** - View and search environment variables
- **Deployment Scripts** - Associate and manage deployment scripts with containers
- **Script Editor** - Edit env vars, ports, volumes, and network settings visually

### Rsync File Synchronization
- **Bidirectional Sync** - Push files to remote or pull files from remote
- **Interactive File Browser** - Navigate local and remote filesystems visually
- **Compression Toggle** - Enable/disable rsync compression on the fly
- **Path Completion** - Type paths directly or browse to select

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
- **Tags** - Sort by first tag alphabetically (hosts without tags appear last)

[↑ Back to Top](#table-of-contents)

---

## Docker Container Management

sshing includes powerful Docker container management capabilities, allowing you to manage containers on any remote server directly from the TUI.

### Entering Docker Mode

1. Select a host from the main table
2. Press `d` to enter Docker mode
3. sshing will SSH to the server and fetch container information
4. The container list view will appear showing all containers

> **Note:** Docker must be installed on the remote server. When entering Docker mode, you'll be prompted whether to use `sudo` for Docker commands. Choose **Yes** if Docker requires root privileges on the server, or **No** if the server has a docker group configured for non-root access. This choice applies to all Docker commands during the session.

### Container List View

The container list displays:
- **Status indicator** - `● Up` (running), `○ Down` (stopped), `✗ Failed` (exited with error)
- **Container name** - The name of the container
- **Image** - The Docker image (shortened for display)
- **Ports** - Port mappings
- **Script** - Whether a deployment script is associated (`✓ has script` or `✗ no script`)

Navigate with `j/k` or arrow keys. The title bar shows scroll position when the list exceeds screen height.

### Container Actions

From the container list, you can perform these actions:

| Key | Action | Description |
|-----|--------|-------------|
| `S` | Start | Start a stopped container |
| `s` | Stop | Stop a running container |
| `r` | Restart | Restart the container |
| `p` | Pull | Pull the latest image for the container |
| `d` | Delete | Remove the container (with confirmation) |
| `X` | Purge | Remove container AND its image |

### Container Inspection Tools

sshing provides several tools to inspect running containers:

#### Log Viewer (`l`)
- View container logs in real-time
- Press `f` to toggle follow mode (live updates)
- Press `+`/`-` to increase/decrease line count (100 → 500 → 1000 → 5000 → 50000)
- Scroll with `j/k`, `g/G`, `Ctrl+d/u`

#### Stats Viewer (`D`)
- View real-time CPU and memory usage
- Visual bar graphs show resource utilization
- Auto-refreshes every few seconds

#### Process Viewer (`T`)
- View running processes inside the container (equivalent to `docker top`)
- Shows PID, user, CPU%, memory%, and command
- Navigate through processes with `j/k`

#### Container Inspect (`I`)
- Deep inspection of container configuration
- View ports, volumes, networks, and full configuration
- Navigate sections with `j/k`

#### Environment Inspector (`E`)
- View all environment variables in the container
- Search/filter variables by typing
- Compare with deployment script variables

### Deployment Scripts

A key feature of sshing's Docker integration is the ability to associate **deployment scripts** with containers. These are shell scripts (typically containing `docker run` or `docker create` commands) that define how a container should be deployed.

#### Why Deployment Scripts?

- **Reproducibility** - Store your exact container configuration as a script
- **Version control** - Keep scripts in your project repository
- **Easy redeployment** - Run the script to recreate the container with the same settings

#### Browsing for Scripts (`b`)

1. Press `b` from the container list to open the file browser
2. Navigate to find your deployment script on the remote server
3. Common locations: project directories, `/opt`, home directories
4. Press `Enter` to select a script

sshing looks for scripts matching patterns like:
- `start*.sh`, `deploy*.sh`, `run*.sh`, `docker*.sh`
- Scripts in common project directories

#### Script Viewer (`v`)

Once a script is associated:
- Press `v` to view the full script content
- See the parsed configuration (env vars, ports, volumes, network)

#### Script Editor (`e`)

Edit deployment scripts visually:
- **Env Vars tab** - Add, edit, or remove environment variables
- **Ports tab** - Manage port mappings
- **Volumes tab** - Configure volume mounts
- **Network tab** - Set network mode

Navigate tabs with `Tab`/`Shift+Tab`, edit values with `Enter`, save with `Ctrl+S`.

#### Running Scripts (`x`)

Press `x` to execute the deployment script, which will:
1. Stop and remove the existing container (if running)
2. Run the deployment script to create a new container
3. Refresh the container list

#### Replacing Containers (`b`)

Press `b` on a container with an existing script to browse for a different script, replacing the association.

[↑ Back to Top](#table-of-contents)

---

## Rsync File Synchronization

sshing includes an interactive rsync interface for synchronizing files between your local machine and remote servers.

### Entering Rsync Mode

1. Select a host from the main table
2. Press `r` to enter rsync mode
3. Configure source and destination paths
4. Execute the sync

> **Note:** Rsync must be installed on both your local machine and the remote server. If rsync is not available locally, the `r` key will be greyed out in the footer.

### Rsync Interface

The rsync view shows:
- **Source path** - Where files will be copied FROM (labeled `[local]` or `[remote]`)
- **Destination path** - Where files will be copied TO
- **Direction indicator** - Shows `Local → Remote` or `Remote → Local`
- **Compression status** - Whether `-z` flag is enabled

### Navigation & Controls

| Key | Action |
|-----|--------|
| `j` / `↓` | Move to next field |
| `k` / `↑` | Move to previous field |
| `i` / `Enter` | Edit the selected field |
| `r` | Toggle sync direction (push/pull) |
| `z` | Toggle compression |
| `b` | Open file browser for current field |
| `Space` | Execute rsync |
| `Esc` / `q` | Return to host list |

### Using the File Browser

Instead of typing paths manually, press `b` to open an interactive file browser:

1. The browser opens showing the appropriate filesystem:
   - **Source field + pushing to remote** → Local filesystem
   - **Source field + pulling from remote** → Remote filesystem
   - **Dest field** → Opposite of source

2. Navigate with:
   - `j/k` or arrows to move through entries
   - `Enter` to enter a directory or select a file
   - `Space` to select the current directory as the path
   - `Backspace` or `h` to go up one directory
   - `g/G` to jump to top/bottom
   - `Esc` to cancel

3. The selected path is inserted into the field

### Executing Rsync

1. Configure your source and destination paths
2. Toggle direction with `r` if needed (default: push to remote)
3. Enable compression with `z` for slow connections
4. Press `Space` to execute

sshing will run rsync with:
- `-avz` flags (archive, verbose, compress if enabled)
- Proper SSH connection using the host's configuration
- Progress displayed in the terminal

After completion, you'll return to the rsync view with a status message.

### Example Workflows

**Deploy local files to server:**
1. Press `r` on a host
2. Set source: `/home/user/project/dist/`
3. Set dest: `/var/www/html/`
4. Ensure direction shows `Local → Remote`
5. Press `Space` to sync

**Download logs from server:**
1. Press `r` on a host
2. Press `r` to toggle direction to `Remote → Local`
3. Press `b` to browse, navigate to `/var/log/myapp/`
4. Set local dest: `/home/user/logs/`
5. Press `Space` to sync

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
| `D` | Delete selected host |
| `d` | Enter Docker mode |
| `r` | Enter Rsync mode |
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

### Docker Container List

| Key | Action |
|-----|--------|
| `j` / `k` / `↑` / `↓` | Navigate containers |
| `g` / `G` | Jump to first/last container |
| `Ctrl+d` / `Ctrl+u` | Page down/up |
| `S` | Start container |
| `s` | Stop container |
| `r` | Restart container |
| `p` | Pull latest image |
| `d` | Delete container |
| `X` | Purge container and image |
| `l` | View logs |
| `D` | View stats |
| `T` | View processes (top) |
| `I` | Inspect container |
| `E` | View environment variables |
| `b` | Browse for deployment script |
| `n` | Create new script |
| `v` | View associated script |
| `e` | Edit associated script |
| `x` | Execute deployment script |
| `Esc` | Return to host list |

### Docker Log Viewer

| Key | Action |
|-----|--------|
| `j` / `k` / `↑` / `↓` | Scroll logs |
| `g` / `G` | Jump to top/bottom |
| `Ctrl+d` / `Ctrl+u` | Page down/up |
| `f` | Toggle follow mode |
| `+` / `=` | Increase line count |
| `-` | Decrease line count |
| `Esc` | Return to container list |

### Docker Script Editor

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Switch between tabs |
| `j` / `k` / `↑` / `↓` | Navigate items |
| `Enter` | Edit selected item |
| `a` / `n` | Add new item |
| `d` | Delete selected item |
| `Ctrl+S` | Save script |
| `Esc` | Cancel / Return |

### Rsync Mode

| Key | Action |
|-----|--------|
| `j` / `k` / `↑` / `↓` | Navigate fields |
| `i` / `Enter` | Edit current field |
| `r` | Toggle sync direction |
| `z` | Toggle compression |
| `b` | Open file browser |
| `Space` | Execute rsync |
| `Esc` / `q` | Return to host list |

### File Browser (Rsync & Docker)

| Key | Action |
|-----|--------|
| `j` / `k` / `↑` / `↓` | Navigate entries |
| `g` / `G` | Jump to first/last entry |
| `Ctrl+d` / `Ctrl+u` | Page down/up |
| `Enter` | Enter directory / Select file |
| `Space` | Select current directory |
| `Backspace` / `h` | Go up one directory |
| `Esc` | Cancel |

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
