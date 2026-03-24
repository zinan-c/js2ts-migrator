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
