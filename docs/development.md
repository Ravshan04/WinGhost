# Development guide

## Prerequisites

- Windows 11 or a supported Windows 10 version with ConPTY.
- Visual Studio Build Tools with the Desktop development with C++ workload.
- Rust 1.88 or newer using the MSVC host toolchain.
- Git and PowerShell 7 are recommended.

## Build and test

```powershell
cargo build --workspace
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

## Working agreements

- Keep platform calls inside adapter crates.
- Represent user actions as commands rather than calling subsystems from UI handlers.
- Make terminal-core behavior deterministic and independent of wall-clock time.
- Add limits for all input-controlled allocations and escape-sequence payloads.
- Benchmark before and after performance-sensitive changes.

## Adding dependencies

Prefer the standard library and narrow crates with active maintenance. A pull
request adding a dependency should explain why it is needed, its license, whether
it introduces native code or unsafe code, and how upgrades will be managed.

## Definition of done

A change is complete when it builds on Windows, includes appropriate tests,
passes formatting and lint checks, updates relevant docs, and describes any known
compatibility or security impact.
