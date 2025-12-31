/// SSH flag option with description
#[derive(Debug, Clone)]
pub struct SshFlagOption {
    pub flag: &'static str,
    pub description: &'static str,
}

/// Shell option with description
#[derive(Debug, Clone)]
pub struct ShellOption {
    pub name: &'static str,
    pub description: &'static str,
}

/// Get all available SSH flags with descriptions
pub fn get_ssh_flag_options() -> Vec<SshFlagOption> {
    vec![
        SshFlagOption {
            flag: "-t",
            description: "Force pseudo-terminal (needed for interactive shells)",
        },
        SshFlagOption {
            flag: "-A",
            description: "Enable SSH agent forwarding",
        },
        SshFlagOption {
            flag: "-X",
            description: "Enable X11 forwarding",
        },
        SshFlagOption {
            flag: "-Y",
            description: "Enable trusted X11 forwarding",
        },
        SshFlagOption {
            flag: "-C",
            description: "Enable compression",
        },
        SshFlagOption {
            flag: "-v",
            description: "Verbose mode (debug connection)",
        },
        SshFlagOption {
            flag: "-vv",
            description: "More verbose (extra debugging)",
        },
        SshFlagOption {
            flag: "-vvv",
            description: "Very verbose (maximum debugging)",
        },
        SshFlagOption {
            flag: "-N",
            description: "Don't execute command (port forwarding only)",
        },
        SshFlagOption {
            flag: "-f",
            description: "Go to background before command execution",
        },
        SshFlagOption {
            flag: "-q",
            description: "Quiet mode (suppress warnings)",
        },
        SshFlagOption {
            flag: "-4",
            description: "Force IPv4 only",
        },
        SshFlagOption {
            flag: "-6",
            description: "Force IPv6 only",
        },
    ]
}

/// Get all available shell options with descriptions
pub fn get_shell_options() -> Vec<ShellOption> {
    vec![
        ShellOption {
            name: "bash",
            description: "Bourne Again Shell (most common)",
        },
        ShellOption {
            name: "zsh",
            description: "Z Shell (enhanced Bourne shell)",
        },
        ShellOption {
            name: "fish",
            description: "Friendly Interactive Shell",
        },
        ShellOption {
            name: "sh",
            description: "Bourne Shell (POSIX standard)",
        },
        ShellOption {
            name: "ksh",
            description: "Korn Shell",
        },
        ShellOption {
            name: "tcsh",
            description: "TENEX C Shell",
        },
        ShellOption {
            name: "dash",
            description: "Debian Almquist Shell",
        },
    ]
}
