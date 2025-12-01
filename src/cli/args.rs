use clap::Parser;
use std::path::PathBuf;

const LONG_ABOUT: &str = concat!(
    "Just Compress Zip - A unified compression utility\n\n",
    "Version: ",
    env!("CARGO_PKG_VERSION"),
    "\n",
    "Author: ",
    env!("CARGO_PKG_AUTHORS"),
    "\n",
    "License: ",
    env!("CARGO_PKG_LICENSE"),
    "\n",
    "Repository: ",
    env!("CARGO_PKG_REPOSITORY"),
    "\n\n",
    "A command-line tool that provides a consistent interface for multiple\n",
    "compression formats including GZIP, BZIP2, XZ, ZIP, TAR, and compound\n",
    "formats (TGZ, TBZ2, TXZ)."
);

const AFTER_HELP: &str = "\
COMPRESSION COMMANDS:
  gzip    GZIP compression (.gz)
  bzip2   BZIP2 compression (.bz2)
  xz      XZ compression (.xz)
  zip     ZIP compression (.zip)
  tar     TAR archive (.tar)
  tgz     TAR + GZIP (.tar.gz)
  tbz2    TAR + BZIP2 (.tar.bz2)
  txz     TAR + XZ (.tar.xz)

EXAMPLES:
  # Compress a file with GZIP
  jcz -c gzip file.txt

  # Compress with BZIP2 at level 9
  jcz -c bzip2 -l 9 file.txt

  # Create compressed archive
  jcz -c tgz directory/

  # Compress with timestamp
  jcz -c gzip -t 2 file.txt

  # Compress and move to directory
  jcz -c gzip -C /backups/ file.txt

  # Collect multiple files into archive
  jcz -c tgz -a myarchive file1.txt file2.txt dir/

  # Decompress any supported format
  jcz -d archive.tar.gz

  # Decompress to specific directory
  jcz -d archive.tar.gz -C /output/

  # Decompress multiple files
  jcz -d file1.gz file2.bz2 file3.xz

  # Force overwrite without prompting
  jcz -d -f archive.tar.gz

ENVIRONMENT VARIABLES:
  JCDBG    Control logging verbosity (error, warn, info, debug)

For more information, visit: https://github.com/saimizi/jc";

#[derive(Parser, Debug)]
#[command(name = "jcz")]
#[command(author = "JCZ Contributors")]
#[command(version)]
#[command(about = "Just Compress Zip - A unified compression utility")]
#[command(long_about = LONG_ABOUT)]
#[command(after_help = AFTER_HELP)]
pub struct CliArgs {
    /// Decompress mode
    #[arg(short = 'd', long)]
    pub decompress: bool,

    /// Force overwrite without prompting
    #[arg(short = 'f', long)]
    pub force: bool,

    /// Compression command (see COMPRESSION COMMANDS below)
    #[arg(short = 'c', long, default_value = "tgz")]
    pub command: String,

    /// Compression level (1-9)
    #[arg(short = 'l', long, default_value = "6")]
    pub level: u8,

    /// Move output to specified directory (works for both compression and decompression)
    #[arg(short = 'C', long)]
    pub move_to: Option<PathBuf>,

    /// Collect files into archive (with parent directory)
    #[arg(short = 'a', long)]
    pub collect: Option<String>,

    /// Collect files into archive (flat, without parent directory)
    #[arg(short = 'A', long)]
    pub collect_flat: Option<String>,

    /// Timestamp option: 0=none, 1=date, 2=datetime, 3=nanoseconds
    #[arg(short = 't', long, default_value = "0")]
    pub timestamp: u8,

    /// Input files or directories
    #[arg(required = true)]
    pub inputs: Vec<PathBuf>,
}

impl CliArgs {
    /// Validate arguments
    pub fn validate(&self) -> Result<(), String> {
        // Validate timestamp option
        if self.timestamp > 3 {
            return Err(format!("Invalid timestamp option: {}", self.timestamp));
        }

        // Validate compression command
        let valid_commands = ["gzip", "bzip2", "xz", "tar", "zip", "tgz", "tbz2", "txz"];
        if !valid_commands.contains(&self.command.as_str()) {
            return Err(format!("Invalid compression command: {}", self.command));
        }

        // Check that collect and collect_flat are not both specified
        if self.collect.is_some() && self.collect_flat.is_some() {
            return Err("Cannot specify both -a and -A".to_string());
        }

        Ok(())
    }
}
