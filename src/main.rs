mod cli;
mod migrator;
mod processor;

use clap::Parser;

fn main() {
    let args = cli::Cli::parse();

    if let Err(err) = processor::run(&args.input, &args.output, args.recursive) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
