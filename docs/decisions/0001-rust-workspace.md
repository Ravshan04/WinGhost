# ADR 0001: Start with a Rust workspace

- Status: accepted
- Date: 2026-09-17

## Context

The project needs a deterministic terminal core, a narrow Windows process adapter,
and a renderer that can evolve independently. Process output is untrusted and the
application will eventually coordinate I/O, parsing, rendering, and UI threads.

## Decision

Use a Rust workspace with separate crates for terminal state, Windows PTY access,
rendering, and application composition. Keep Windows APIs behind the `pty-windows`
boundary. Defer the final UI shell and GPU API choice until focused prototypes can
be compared against accessibility, IME, packaging, latency, and maintenance needs.

## Consequences

- Core logic can be tested without a window or Windows process.
- Unsafe platform interop can be isolated and reviewed.
- The project avoids prematurely committing to a UI or rendering framework.
- The first prototype must still validate Rust integration with the chosen native APIs.
