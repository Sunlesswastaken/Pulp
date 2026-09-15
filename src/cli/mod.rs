use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "pulp",
    version,
    about = "A fast PDF toolkit for the terminal",
    long_about = "A fast, native PDF toolkit.\n\nRun `pulp` with no arguments to open the interactive terminal UI, or pass a subcommand to use pulp as a plain CLI tool."
)]
pub struct Cli {
    /// Theme to use for the interactive UI. Bundled choices: opencode
    /// (default), dracula, everforest, flexoki, gruvbox, kanagawa, monokai,
    /// nord, one-dark, palenight, rosepine, solarized, tokyonight — or any
    /// theme JSON placed in ~/.config/pulp/themes/.
    #[arg(long, value_name = "THEME")]
    pub theme: Option<String>,

    /// Subcommand to run; omitting it opens the TUI.
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Shrink a PDF's file size — the headline operation
    Compress {
        /// Input PDF file
        file: PathBuf,
    },

    /// Combine several PDFs into a single document
    Merge {
        /// PDFs to merge, in order
        files: Vec<PathBuf>,

        /// Output file
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Split a PDF into separate files
    Split {
        /// Input PDF file
        file: PathBuf,
    },

    /// Remove pages from a PDF
    Remove {
        /// Input PDF file
        file: PathBuf,
    },

    /// Extract pages from a PDF into a new file
    Extract {
        /// Input PDF file
        file: PathBuf,
    },

    /// Add or change a password on a PDF
    Password {
        /// Input PDF file
        file: PathBuf,
    },

    /// Show information about a PDF
    Info {
        /// Input PDF file
        file: PathBuf,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_arguments_means_tui() {
        let cli = Cli::try_parse_from(["pulp"]).unwrap();
        assert!(cli.command.is_none());
    }

    #[test]
    fn compress_parses_input_file() {
        let cli = Cli::try_parse_from(["pulp", "compress", "a.pdf"]).unwrap();
        match cli.command.unwrap() {
            Command::Compress { file } => assert_eq!(file, PathBuf::from("a.pdf")),
            other => panic!("unexpected command {other:?}"),
        }
    }

    #[test]
    fn merge_parses_files_and_output() {
        let cli =
            Cli::try_parse_from(["pulp", "merge", "a.pdf", "b.pdf", "-o", "out.pdf"]).unwrap();
        match cli.command.unwrap() {
            Command::Merge { files, output } => {
                assert_eq!(files, vec![PathBuf::from("a.pdf"), PathBuf::from("b.pdf")]);
                assert_eq!(output, PathBuf::from("out.pdf"));
            }
            other => panic!("unexpected command {other:?}"),
        }
    }

    #[test]
    fn unknown_subcommand_is_rejected() {
        assert!(Cli::try_parse_from(["pulp", "squash"]).is_err());
    }

    #[test]
    fn theme_flag_parses() {
        let cli = Cli::try_parse_from(["pulp", "--theme", "nord"]).unwrap();
        assert_eq!(cli.theme.as_deref(), Some("nord"));
        assert!(cli.command.is_none());
    }

    #[test]
    fn theme_flag_defaults_to_none() {
        let cli = Cli::try_parse_from(["pulp"]).unwrap();
        assert_eq!(cli.theme, None);
    }
}
