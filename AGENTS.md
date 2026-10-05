# Repository Guidelines

## Project Structure

Samsung TV Remote is a responsive native macOS remote written in Rust. The
repository is currently an early starting point, with a minimal Cargo package
and no application behavior or tests yet. Keep the root for project metadata
and top-level configuration. Put application code in `src/`, integration tests in
`tests/`, and bundled resources in `assets/`. Keep network protocol, device
discovery, and macOS UI responsibilities in focused modules.

## Development Workflow

The canonical Cargo commands, local validation workflow, and commit and pull
request conventions live in [`docs/contribution-guide.md`](docs/contribution-guide.md).
Do not claim a command works until its configuration is committed and it has been
run successfully.

## Coding Style and Naming

Use stable Rust and let `rustfmt` determine formatting; use four spaces and no
tabs. Follow Rust naming: `snake_case` for modules, functions, and variables;
`PascalCase` for types and traits; `SCREAMING_SNAKE_CASE` for constants. Prefer
explicit error types and small public interfaces. Add comments only where
intent or a non-obvious Samsung protocol decision needs explanation.

## Testing Guidelines

Add tests with new behavior and bug fixes. Mirror the source layout under
`tests/` and use test names that state the expected result, such as
`connects_when_tv_accepts_pairing`. Cover protocol parsing, pairing failures,
and network-boundary behavior with deterministic fakes rather than real TVs.
Run the checks in the contribution guide before opening a pull request.
