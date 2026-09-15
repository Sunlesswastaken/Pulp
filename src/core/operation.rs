use std::fmt;

/// The operations pulp can perform.
///
/// This is the single vocabulary shared by the CLI subcommands and the
/// TUI navigation list, so both interfaces stay in lockstep.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operation {
    Compress,
    Merge,
    Split,
    Remove,
    Extract,
    Password,
    Info,
}

impl Operation {
    pub const ALL: [Operation; 7] = [
        Operation::Compress,
        Operation::Merge,
        Operation::Split,
        Operation::Remove,
        Operation::Extract,
        Operation::Password,
        Operation::Info,
    ];

    /// Human-facing title, e.g. "Remove pages".
    pub fn title(&self) -> &'static str {
        match self {
            Operation::Compress => "Compress",
            Operation::Merge => "Merge",
            Operation::Split => "Split",
            Operation::Remove => "Remove pages",
            Operation::Extract => "Extract pages",
            Operation::Password => "Password",
            Operation::Info => "Info",
        }
    }

    /// Short url-style identifier, used in breadcrumbs and paths.
    pub fn slug(&self) -> &'static str {
        match self {
            Operation::Compress => "compress",
            Operation::Merge => "merge",
            Operation::Split => "split",
            Operation::Remove => "remove",
            Operation::Extract => "extract",
            Operation::Password => "password",
            Operation::Info => "info",
        }
    }

    /// One-line description shown as secondary text under the title.
    pub fn description(&self) -> &'static str {
        match self {
            Operation::Compress => "Reduce file size",
            Operation::Merge => "Combine documents",
            Operation::Split => "Separate into pieces",
            Operation::Remove => "Delete pages",
            Operation::Extract => "Copy pages to a new file",
            Operation::Password => "Encrypt the document",
            Operation::Info => "Inspect metadata",
        }
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.title())
    }
}
