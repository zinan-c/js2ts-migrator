use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "js2ts-migrator")]
#[command(about = "Basic .js to .ts file copy scaffold", long_about = None)]
pub struct Cli {
    /// Input .js file or directory
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output directory for generated .ts files
    #[arg(short, long)]
    pub output: PathBuf,

    /// Recurse into subdirectories when input is a directory
    #[arg(short, long, default_value_t = false)]
    pub recursive: bool,

    /// Preview without writing output files
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}
