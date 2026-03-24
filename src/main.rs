mod cli;
mod file_processor;
mod migrator;
mod processor;
mod server;

use clap::Parser;

#[tokio::main]
async fn main() {
    let args = cli::Cli::parse();

    if args.serve {
        if let Err(err) = server::run(args.port).await {
            eprintln!("error: {err}");
            std::process::exit(1);
        }
        return;
    }

    let input = match args.input {
        Some(path) => path,
        None => {
            eprintln!("error: --input is required unless --serve is used");
            std::process::exit(1);
        }
    };
    let output = match args.output {
        Some(path) => path,
        None => {
            eprintln!("error: --output is required unless --serve is used");
            std::process::exit(1);
        }
    };

    file_processor::set_dry_run(args.dry_run);

    if let Err(err) = processor::run(&input, &output, args.recursive) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
