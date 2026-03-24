mod cli;
mod file_processor;
mod migrator;
mod processor;

use clap::Parser;

fn main() {
    let args = cli::Cli::parse();

    file_processor::set_dry_run(args.dry_run);

    if let Err(err) = processor::run(&args.input, &args.output, args.recursive) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
