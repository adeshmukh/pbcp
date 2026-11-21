# pbcp implementation plan

## Product goal

`pbcp` is a universal, dependency-free command-line utility that copies data
from a shell pipeline to the clipboard associated with the user operating the
terminal.

The motivating workflows are:

```sh
command | pbcp
pbcp < file.txt
```

The highest-priority environments are:

1. WSL copying to the Windows host clipboard.
2. A Linux shell reached over SSH copying to the clipboard of the local
   terminal.
3. Sessions nested inside `tmux`.
4. Local Linux terminals.
5. macOS, so `pbcp` can replace `pbcopy` in portable muscle memory and scripts.

GNU `screen` support is desirable when it can share the terminal transport
design without substantial complexity. Native Windows-shell usage is useful
eventually but is not an initial priority.

## Initial scope

The first release is intentionally copy-only:

```text
stdin -> pbcp -> selected clipboard backend
```

Reading the clipboard is deferred. Remote clipboard reads are frequently
unsupported or blocked for security reasons, whereas copy via OSC 52 directly
serves the motivating SSH workflow.

The first release should provide:

- Input from standard input.
- Automatic backend and transport selection.
- `--backend=<name>` to force a backend.
- `--diagnose` to explain relevant environment detection and the backend that
  would be selected without copying clipboard content.
- Clear nonzero exits and actionable errors.
- Byte-level tests for Base64, OSC 52, and multiplexer wrapping.

Literal command-line arguments, clipboard paste, rich clipboard formats, and
images are out of scope until the copy pipeline is reliable.

## Dependency policy

“Dependency-free” means:

- No third-party crates in the default Cargo build.
- No required installation of `base64`, `xclip`, `xsel`, `wl-copy`, PowerShell
  modules, or similar helpers.
- One deployable `pbcp` executable per target.
- Standard OS APIs, system libraries, terminal protocols, and facilities
  guaranteed by the host environment are allowed.
- WSL may use the built-in Windows interoperability path and `clip.exe` as a
  fallback because it is part of Windows, not an optional package.

Optional compatibility backends that execute non-guaranteed tools may be
considered later, but the normal build and primary workflows must not require
them.

## Architecture

Keep protocol construction separate from environment detection and side
effects:

```text
CLI and input
    |
backend selection
    |
clipboard backend
    |
terminal transport or native platform API
```

Suggested Rust module boundaries:

```text
src/
  main.rs
  cli.rs
  input.rs
  detect.rs
  base64.rs
  backend/
    mod.rs
    osc52.rs
    wsl.rs
    macos.rs
    wayland.rs
    x11.rs
    windows.rs
  terminal/
    mod.rs
    direct.rs
    tmux.rs
    screen.rs
```

This layout is a guide, not a requirement. Prefer fewer modules until separation
is useful.

### OSC 52

OSC 52 is the first backend because it addresses both primary needs:

- WSL applications can ask a compatible Windows terminal to update the Windows
  clipboard.
- Programs on an SSH host can send the sequence through SSH to the user's local
  terminal.

Protocol generation must:

- Base64-encode internally.
- Produce exact, unit-testable bytes without writing to a terminal.
- Write the final control sequence to the controlling terminal, normally
  `/dev/tty`, rather than standard output.
- Correctly pass through `tmux`, and through `screen` when supported.
- Avoid printing clipboard content in diagnostics or errors.
- Detect unsupported conditions where practical, while allowing an explicit
  `--backend=osc52` override because terminal capability cannot always be
  inferred reliably.

### WSL fallback

When OSC 52 is unavailable or explicitly bypassed, WSL may invoke the built-in
Windows `clip.exe`. Encoding behavior must be tested with ASCII, Unicode, empty
input, and input without a trailing newline. The fallback must report process
startup and nonzero-exit failures.

### Native platform backends

- **macOS:** use the native pasteboard API in a small platform-gated module.
  Calling `pbcopy` may be a temporary compatibility path but is not the desired
  final backend.
- **Wayland:** implement the required clipboard protocol and process-lifetime
  behavior without requiring `wl-copy`.
- **X11:** implement selection ownership and data serving without requiring
  `xclip` or `xsel`.
- **Windows:** use the Win32 clipboard API when native Windows-shell support is
  prioritized.

Linux graphical clipboard ownership can require the provider process to remain
available after the original command exits. That lifecycle must be designed and
tested rather than hidden behind a premature abstraction.

## Backend-selection principles

The exact automatic order should be finalized with executable tests. The
selection policy must follow these principles:

1. An explicit `--backend` wins or fails clearly.
2. An SSH session should normally prefer OSC 52 so the clipboard belongs to the
   local operator, not the remote graphical session.
3. WSL should prefer a verified terminal path and retain `clip.exe` as a host
   fallback.
4. A local graphical Linux session may use Wayland or X11 when its native
   backend exists.
5. Automatic selection must not report success merely because an environment
   variable was present.

All detection inputs should be represented by a testable environment structure
rather than read opportunistically throughout the program.

## Safety and correctness

- Bound allocations and enforce the default 64 KiB raw-input limit before
  release. Configuration sources, finite overrides, and warning/error behavior
  are specified in [CONFIGURATION.md](CONFIGURATION.md).
- Decide and document whether NUL-containing or arbitrary binary input is
  rejected. The initial UX is text-oriented, but validation must not corrupt
  input silently.
- Treat terminal escape generation as security-sensitive output.
- Do not include copied content in logs, diagnostics, panic messages, or process
  listings.
- Keep unsafe code limited to native FFI boundaries.
- Test short writes and I/O failures where the standard library interface makes
  them observable.

## Cross-platform development

Most core work can be completed from WSL:

- Pure protocol and selection logic can be fully unit-tested.
- `rustup` targets can type-check platform-gated code with `cargo check
  --target ...` where target standard libraries are available.
- Windows GNU binaries can be cross-compiled and run through WSL
  interoperability when needed.

A genuine macOS environment is eventually required to link and behavior-test
the native pasteboard backend. Use either a normal clone on a Mac or a hosted
macOS GitHub Actions runner after the repository has a remote. Agent worktrees
on the WSL host provide edit isolation, not additional operating systems.

## Milestones

### M0: Project contract

- Repository agent guidance.
- Durable implementation plan.
- Resumable progress tracker.

### M1: Tested core

- Zero-dependency Cargo binary project.
- CLI parsing sufficient for initial options.
- Input handling.
- Internal Base64 implementation.
- Pure OSC 52 sequence generation with edge-case tests.

### M2: Terminal transports

- Direct controlling-terminal output.
- `tmux` passthrough with exact-byte tests.
- `screen` passthrough if it remains a contained addition.
- Environment detection and `--diagnose`.

### M3: WSL and SSH vertical slice

- Automatic WSL/SSH backend policy.
- WSL `clip.exe` fallback.
- Manual WSL → Windows clipboard validation.
- Manual WSL-inside-`tmux` validation.
- Manual remote SSH → local clipboard validation.

### M4: Product hardening

- Configuration discovery, per-key precedence, CLI overrides, and diagnostics
  according to [CONFIGURATION.md](CONFIGURATION.md).
- Payload limits and binary-input policy.
- Actionable errors and stable exit behavior.
- User-facing README and installation instructions.
- Release builds and checks.

### M5: macOS

- Native pasteboard implementation.
- macOS build and behavior validation.

### M6: Local graphical Linux

- Native Wayland backend and lifecycle.
- Native X11 backend and lifecycle.

### M7: Native Windows and broader distribution

- Native Win32 backend.
- Release matrix and packaging as justified by usage.

## Definition of done for the first useful release

The initial release is complete when:

- `command | pbcp` copies Unicode text from WSL into the Windows clipboard.
- The same command works from a remote Linux host over SSH in a documented,
  supported terminal.
- Direct and `tmux`-nested paths are validated; `screen` status is explicit.
- The default Cargo build has no third-party dependencies.
- Automatic selection, forced selection, protocol bytes, Unicode, empty input,
  and failure paths have focused tests.
- Diagnostics identify the chosen path without exposing clipboard contents.
- Limitations and supported terminal assumptions are documented.
