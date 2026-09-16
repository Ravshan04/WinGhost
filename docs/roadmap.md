# Roadmap

Dates are intentionally omitted until the first performance and integration
spikes establish realistic estimates.

## Milestone 0 — Foundations

- [x] Create the original project identity and repository scaffold.
- [x] Define initial module boundaries and contribution rules.
- [x] Add Windows CI and a manual release-artifact placeholder.
- [ ] Choose a permanent project name after availability and trademark review.
- [ ] Record architecture decisions for the UI shell and GPU stack.

Exit condition: a new contributor can clone, build, test, and understand the
intended system boundaries.

## Milestone 1 — Interactive vertical slice

- [ ] Create, resize, and close a ConPTY session safely.
- [ ] Launch a configured PowerShell profile.
- [ ] Parse a deliberately small VT subset into a screen grid.
- [ ] Open a native Windows window and render monospace text.
- [ ] Send keyboard input to the child process.
- [ ] Add end-to-end smoke tests for startup and clean shutdown.

Exit condition: a developer can open one window, run commands, resize it, and exit.

## Milestone 2 — Terminal correctness

- [ ] Expand VT/ANSI coverage using documented compatibility fixtures.
- [ ] Implement scrollback, selection, copy/paste, search, and hyperlinks.
- [ ] Add Unicode grapheme, width, emoji, IME, and bidirectional-text tests.
- [ ] Handle alternate screen, mouse reporting, bracketed paste, and common TUIs.
- [ ] Publish a compatibility matrix and known limitations.

Exit condition: PowerShell, cmd.exe, WSL, SSH, and representative TUIs work reliably.

## Milestone 3 — Windows product experience

- [ ] Add profiles, tabs, split panes, command palette, and key binding editor.
- [ ] Follow system light/dark, scaling, accessibility, and reduced-motion settings.
- [ ] Add font fallback, themes, background opacity, and careful acrylic support.
- [ ] Define configuration schema, validation, migration, and live reload.
- [ ] Add crash recovery without silently restoring sensitive session contents.

Exit condition: the application is comfortable as a daily terminal for early adopters.

## Milestone 4 — Preview distribution

- [ ] Produce signed MSIX and portable archives.
- [ ] Add opt-in update checks and release provenance.
- [ ] Run startup, latency, memory, and rendering benchmarks in CI.
- [ ] Complete security review and threat model.
- [ ] Publish preview documentation, troubleshooting, and migration guidance.

Exit condition: repeatable, signed preview releases with actionable telemetry-free
diagnostics and a documented support policy.
