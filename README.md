# js2ts-migrator
Version: 1.1

implement the javascript migrate to typescript via Rust

## What's New in 1.1
- Added per-file processing pipeline (read → parse → infer → generate → write)
- Added `--dry-run` mode for preview without writing
- Added per-file status output with variable count and line count
- Added directory processing tolerance (continue on errors) and summary report
- Added accurate variable declaration counting
- Added output collision warnings in non-recursive mode
- Added support for `.js`, `.jsx`, `.mjs`, `.cjs` inputs

## Usage
Single file:
```sh
cargo run -- --input path/to/file.js --output out
```

Directory (non-recursive):
```sh
cargo run -- --input path/to/dir --output out
```

Directory (recursive):
```sh
cargo run -- --input path/to/dir --output out --recursive
```

Dry run (no writes):
```sh
cargo run -- --input path/to/dir --output out --dry-run
```

Web UI + API:
```sh
cargo run -- --serve
```

Then open:
```
http://localhost:8222
```
