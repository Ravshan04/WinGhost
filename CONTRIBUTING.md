# Contributing to WinGhost

WinGhost is at an early stage. Small, focused changes that establish reliable
foundations are more useful than broad feature implementations.

## Before you start

- Search existing issues before opening a new one.
- For architecture changes or large features, open a proposal issue first.
- Keep the project original. Do not submit copied code, assets, names, or visual
  treatments from Ghostty or another terminal unless their license and origin
  are documented and the use has been explicitly accepted by maintainers.
- Never include secrets, private logs, or personal shell history in reports.

## Development setup

1. Install the Rust MSVC toolchain and Visual Studio Build Tools on Windows.
2. Clone the repository.
3. Run the local quality checks:

   ```powershell
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```

## Pull requests

- Keep each pull request centered on one problem.
- Add tests for behavior changes where practical.
- Update documentation when a public interface or architectural decision changes.
- Describe user impact, testing performed, and known limitations.
- Avoid adding dependencies without explaining the maintenance and security cost.

## Commit messages

Use a short, imperative subject, for example: `Add screen buffer resize tests`.
Conventional Commits are welcome but not required.

## Reporting bugs

Include the Windows version, shell/profile, reproduction steps, expected result,
actual result, and a minimal sanitized terminal recording or screenshot when useful.
