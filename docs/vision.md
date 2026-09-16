# Product vision

## The problem

Windows developers often move between PowerShell, Command Prompt, WSL, SSH, and
tool-specific consoles. A terminal should make those transitions fast without
requiring users to trade native Windows behavior for rendering quality or
customization.

## The promise

WinGhost should be a fast, calm, keyboard-first terminal that feels designed for
Windows rather than merely ported to it.

## Target users

- Windows developers who regularly use PowerShell, WSL, Git, and SSH.
- Operators who need reliable keyboard navigation, search, and long-running sessions.
- Users who want modern typography and themes without a complicated setup.

## Experience pillars

### Native by default

Respect Windows windowing, input, accessibility, notifications, jump lists, and
system theme behavior. Prefer familiar platform conventions when they do not
conflict with terminal compatibility.

### Fast enough to disappear

Measure launch time, input-to-present latency, frame pacing, resize behavior, and
memory use. Performance work begins with repeatable benchmarks.

### Powerful, not noisy

Profiles, panes, key bindings, themes, and configuration should be discoverable.
Advanced capabilities should not make the default experience feel busy.

### Trustworthy sessions

Terminal emulation correctness, Unicode behavior, clipboard safety, and recovery
from renderer or shell failures are part of the product, not cleanup work.

## Naming

`WinGhost` is a temporary project name. A permanent name must be original,
searchable, legally reviewable, and not imply affiliation with another terminal.
