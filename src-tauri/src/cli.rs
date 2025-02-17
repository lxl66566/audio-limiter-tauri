use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Make any command automatically run on startup")]
pub struct Cli {
    /// subcommand
    #[command(subcommand)]
    pub command: Option<Commands>,
    /// input device name
    #[arg(short, long)]
    pub input_device: Option<String>,
    /// output device name
    #[arg(short, long)]
    pub output_device: Option<String>,
    /// initial target loudness
    #[arg(short, long)]
    #[arg(default_value = "-20")]
    pub target_loudness: i32,
    /// only for debugging, enable this will break the audio output quality.
    #[arg(short, long)]
    pub debug: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all audio devices
    #[command(visible_alias = "l", visible_alias = "info", visible_alias = "i")]
    List,
}
