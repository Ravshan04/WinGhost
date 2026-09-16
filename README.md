# WinGhost

> **Status:** early design and scaffolding. There is no usable terminal build yet.

WinGhost is a working name for an original, Windows-first terminal emulator. The
project aims to combine fast GPU-rendered text, native Windows integration, and a
focused configuration experience for PowerShell, Command Prompt, WSL, and other
ConPTY-compatible shells.

WinGhost is inspired by the broader modern-terminal product category. It is not
affiliated with Ghostty, does not use Ghostty branding, and does not copy Ghostty
source code or design assets.

## Project goals

- Feel at home on Windows 11, including keyboard, window, theme, and accessibility behavior.
- Use Windows ConPTY as the first process-hosting backend.
- Render text efficiently on the GPU while preserving correctness and readability.
- Support tabs, split panes, profiles, search, links, and configurable key bindings.
- Keep terminal parsing and state independent from the Windows UI and renderer.
- Make startup, input latency, memory use, and compatibility measurable.

## Non-goals for the first release

- Cross-platform support.
- Shell, multiplexer, or remote-login implementations.
- Plugin APIs before the core terminal behavior is stable.
- Pixel-for-pixel reproduction of another terminal.

## Repository layout

```text
apps/winghost/          Minimal application entry point
crates/terminal-core/   Platform-independent terminal state and parsing
crates/pty-windows/     Windows ConPTY boundary
crates/renderer/        Renderer-facing scene model
docs/                   Product, architecture, development, and roadmap notes
```

## Quick start

The scaffold requires Rust 1.85 or newer. On Windows with the Rust MSVC toolchain:

```powershell
cargo build --workspace
cargo test --workspace
cargo run -p winghost
```

The current binary only confirms that the workspace is wired correctly. It does
not open a terminal window yet. See the [roadmap](docs/roadmap.md) for the path to
the first interactive prototype.

## Principles

1. Correct terminal behavior before visual polish.
2. Measured performance before performance claims.
3. Small, testable modules with explicit boundaries.
4. Accessible defaults and keyboard-first operation.
5. Original implementation, identity, and user experience.

## Contributing

Early contributions are welcome, especially design feedback, Windows terminal
compatibility research, tests, and small roadmap items. Read
[CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.

## License

WinGhost is available under the [MIT License](LICENSE).
