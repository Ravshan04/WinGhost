# Architecture

This document records the initial direction. Interfaces will change while the
first vertical slice is built.

## System shape

```text
Windows input/window events
          |
          v
    Application shell  <---->  Configuration and commands
          |
          +----> ConPTY session ----> PowerShell / cmd / WSL / other shell
          |           |
          |           v
          +----> Terminal core ----> Immutable render snapshot
                                      |
                                      v
                                  GPU renderer
```

## Module boundaries

### `terminal-core`

Owns the terminal state machine: cells, cursor, modes, scrollback, selection,
and eventually VT parsing. It must not depend on Win32, a UI toolkit, or a GPU API.
Its inputs are bytes and user-intent commands; its outputs are state changes and
render snapshots.

### `pty-windows`

Owns the ConPTY lifecycle and process boundary: pseudoconsole creation, pipes,
child process launch, resize, shutdown, and asynchronous byte transport. Unsafe
Win32 calls, when introduced, stay behind a small reviewed API in this crate.

### `renderer`

Consumes immutable snapshots and produces frames. Font shaping, glyph caching,
damage tracking, cursor effects, and GPU resources belong here. It must not own
terminal semantics or shell processes.

### `winghost`

Composes windows, tabs, panes, commands, configuration, sessions, and rendering.
The first scaffold is a console binary; the Windows windowing shell is selected
during the first prototype milestone after a focused spike.

## Data flow

1. The PTY reader passes byte chunks to the terminal core.
2. The core parses input and updates its state on a single ordered event path.
3. The core publishes a cheap, immutable render snapshot.
4. The renderer draws only changed regions when possible.
5. Keyboard and pointer actions become commands before reaching the PTY or core.

Bounded queues should provide backpressure between process I/O, terminal state,
and rendering. Rendering may skip obsolete snapshots; terminal input may not be
reordered or discarded.

## Dependency direction

Lower-level crates never depend on the application shell. Platform-specific code
is isolated at adapter boundaries. The core remains deterministic and testable
with recorded byte streams.

## Initial technology choices

- Rust for memory safety, concurrency, and a portable core boundary.
- Windows ConPTY for local process hosting.
- A GPU abstraction will be chosen by measurement after comparing Direct3D/Direct2D
  integration and `wgpu` in a prototype.
- A native Windows windowing approach will be selected after testing accessibility,
  text input, IME, title-bar integration, and packaging requirements.

## Quality strategy

- Unit tests for parser, screen, cursor, and selection behavior.
- Golden tests using sanitized VT byte streams.
- Integration tests around ConPTY lifecycle and resize on Windows CI.
- Benchmarks for parsing, scrolling, shaping, damage calculation, and startup.
- Manual compatibility matrix across PowerShell, cmd.exe, WSL, SSH, and common TUIs.

## Security boundaries

Treat terminal output as untrusted input. Escape sequence parsing, OSC links,
clipboard access, file paths, shell launch arguments, and configuration imports
require explicit validation and limits. The renderer must not interpret terminal
content as application commands.
