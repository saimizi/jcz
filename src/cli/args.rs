use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "jcz")]
#[command(author = "JCZ Contributors")]
#[command(version)]
#[command(about = "Just Compress Zip - A unified compression utility", long_about = None)]
pub struct CliArgs {
    /// Decompress mode
    #[arg(short = 'd', long)]
    pub decompress: bool,

    /// Force overwrite without prompting
    #[arg(short = 'f', long)]
    pub force: bool,

    /// Compression command
    #[arg(short = 'c', long, default_value = "tgz")]
    pub command: String,

    /// Compression level (1-9)
    #[arg(short = 'l', long, default_value = "6")]
    pub level: u8,

    /// Move compressed file to specified directory
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
