# pbcp project progress

Last updated: 2026-11-11

This file is the canonical handoff between implementation sessions. Keep it
short, factual, and current. `AGENTS.md` defines the required update protocol.

## Current state

**Phase:** M0 complete; M1 complete; M2 complete; M3 blocked.

**Next action:** Validate the OSC 52 sequence from `hop` in the actual local
terminal and confirm the Windows clipboard receives the test text.

**Blockers:** The tool session captures SSH output instead of delivering it to
the user's local terminal emulator, so OSC 52 clipboard handling remains
unverified.

## Milestone status

| Milestone | Status | Notes |
|---|---|---|
| M0 Project contract | Done | Agent guide, implementation plan, and tracker created |
| M1 Tested core | Done | Zero-dependency crate, CLI parsing, stdin, Base64, and OSC 52 core |
| M2 Terminal transports | Done | CLI wiring, direct output, `tmux` and `screen` passthrough, selection, and presence-only `--diagnose` |
| M3 WSL and SSH vertical slice | Blocked | WSL `clip.exe` path validated, including from tmux; `hop` delivered OSC 52 bytes over SSH to a PTY, but local terminal clipboard handling remains unverified |
| M4 Product hardening | Pending | Required before first release |
| M5 macOS | Pending | Requires a Mac or hosted macOS runner for final validation |
| M6 Local graphical Linux | Pending | Wayland and X11 lifecycle work |
| M7 Native Windows/distribution | Pending | Lowest initial priority |

Use only these status values: `Pending`, `In progress`, `Blocked`, `Done`.

## Active task

Validate the OSC 52 sequence from `hop` in the actual local terminal and
confirm the Windows clipboard receives the test text.

## Decisions

- Rust is the implementation language.
- The default build will use no third-party Cargo crates.
- The product is copy-only initially.
- OSC 52 is the first backend because it serves both WSL and remote SSH.
- WSL `clip.exe` is an allowed built-in fallback.
- `tmux` is required for the initial useful release.
- `screen` should be supported when it remains a small transport addition.
- Native macOS support is planned; final validation requires macOS.
- Native Windows-shell support is lower priority than WSL.
- Initial implementation should remain in one worktree until the backend and
  transport contracts stabilize.
- The core reads and encodes arbitrary input bytes without imposing the still
  open NUL/binary-input policy.
- Environment detection captures only non-empty presence signals for SSH,
  WSL, tmux, screen, Wayland, and X11; it does not retain or print variable
  values.
- Backend selection prefers OSC 52 in SSH sessions, including SSH nested in
  WSL; otherwise WSL selects its built-in `clip.exe` fallback. Explicit
  `--backend=osc52` overrides automatic selection, while `--backend=wsl-clip`
  is valid only in WSL. Current environment signals cannot verify terminal OSC
  52 support, so WSL does not automatically select OSC 52.
- Mux passthrough is byte-tested separately for tmux and GNU screen. When both
  environment signals are present, transport is rejected rather than guessing
  the nesting order.
- The WSL `clip.exe` backend accepts UTF-8 text, converts it to UTF-16LE with no
  BOM or added newline, and rejects invalid UTF-8 and NUL bytes rather than
  silently corrupting text. Other backends' binary-input policy remains open.
- GPT-6 Luna is preferred for focused implementation, GPT-6 Sol for correctness
  review and integration, and GPT-6 Astra only for unresolved difficult work.

## Open decisions

- Maximum clipboard payload and override behavior.
- Policy for NUL-containing or otherwise binary input.
- Exact CLI spelling and exit codes beyond `--backend` and `--diagnose`.
- Reliable automatic capability heuristics versus explicit OSC 52 selection.

Resolve these only when the relevant milestone needs them. Record the result in
**Decisions** and in tests or user documentation as appropriate.

## Validation evidence

Validation:

- `cargo test --all-targets` — 21 tests passed.
- `cargo clippy --all-targets -- -D warnings` — passed.
- `cargo tree --edges normal` — only the local `pbcp` package; no third-party
  dependencies.
- `cargo fmt --all -- --check` — passed after formatting.
- `git diff --check` — passed.
- After deterministic selection: formatting, all 26 tests, Clippy, dependency
  tree, and whitespace checks passed.
- M2 completion: `cargo fmt --all -- --check`, `cargo test --all-targets` (35
  tests passed), `cargo clippy --all-targets -- -D warnings`, `cargo tree
  --edges normal`, and `git diff --check` all passed. Automatic and forced
  `--diagnose` smoke checks report signal presence and route without environment
  values. This WSL2 host has `clip.exe`, `tmux`, and `screen`; no clipboard
  contents were inspected.
- At M2 completion, no direct-terminal, tmux-session, WSL, or SSH copy behavior
  had been manually validated.
- M3 checkpoint: `cargo fmt --all -- --check`, `cargo test --all-targets` (42
  unit tests and 1 CLI integration test passed), `cargo clippy --all-targets
  -- -D warnings`, `cargo tree --edges normal`, and `git diff --check` all
  passed. The real WSL `clip.exe` path preserved a Unicode test marker without
  a trailing newline; the same path passed from a tmux pane. The remote binary
  on `hop` selected OSC 52 and delivered the expected bytes to an allocated
  PTY over SSH. The actual local terminal's OSC 52 clipboard handling remains
  unverified because tool output is captured rather than sent to that terminal.

Record commands and outcomes here, distinguishing:

- Unit-tested behavior.
- Cross-compiled or type-checked targets.
- Manually validated operating systems, terminals, and multiplexers.

## Session log

### 2026-11-11 — Planning

- Established product goals, priorities, dependency policy, architecture, and
  milestones.
- Added resumable repository instructions and progress tracking.
- No production code was created.

### 2026-11-11 — M1 tested core

- Added the zero-dependency Rust package, initial option parsing, byte-preserving
  input reader, Base64 encoder, and pure OSC 52 generator.
- Added 14 focused unit tests; test, lint, formatting, and dependency checks
  passed.
- No terminal I/O or platform backend was added.

### 2026-11-11 — M2 transport foundation

- Added direct controlling-terminal writes with explicit errors, pure `tmux`
  passthrough wrapping, and a testable environment-signal snapshot.
- Added short-write, I/O-failure, wrapper-byte, and environment-detection tests;
  21 tests, Clippy, formatting, dependency-tree, and whitespace checks passed.
- Changed: `Cargo.toml`, `Cargo.lock`, and `src/{base64,cli,detect,input,lib,main,osc52}.rs`,
  `src/terminal/{mod,direct,tmux}.rs`.
- The binary entry point is not wired to the library yet; `--diagnose`,
  selection, actual terminal/multiplexer delivery, and WSL/SSH validation remain.

### 2026-11-11 — Deterministic backend selection

- Added a pure backend selector and focused tests for explicit selection, SSH
  precedence, WSL fallback, unsupported choices, and undetected environments.
- Automatic selection chooses OSC 52 for SSH (including SSH inside WSL),
  `wsl-clip` for WSL otherwise, and returns an explicit error when no supported
  environment is detected. No terminal capability verification is inferred
  from presence signals.
- Changed: `src/selection.rs`, `src/lib.rs`, and this progress tracker.
- Formatting, all 26 tests, Clippy, dependency-tree, and whitespace checks
  passed. No platform or terminal behavior was manually validated.
- The selector is not yet wired into the binary; the next task is CLI
  integration and `--diagnose`.

### 2026-11-11 — M2 terminal transports

- Wired CLI execution to backend selection, standard input, OSC 52 generation,
  controlling-terminal output, and selected terminal transport. Implemented
  presence-only `--diagnose` reporting with explicit and automatic selection.
- Added byte-tested GNU screen DCS passthrough alongside tmux; both being
  detected together returns an explicit unsupported-nesting error rather than
  guessing the order.
- Changed: `src/diagnose.rs`, `src/lib.rs`, `src/main.rs`,
  `src/terminal/{mod,screen}.rs`, and this tracker.
- Formatting, all 35 tests, Clippy, dependency tree, whitespace checks, and
  automatic/forced diagnostic smoke checks passed. No OSC 52 clipboard copy
  was manually validated.
- M2 is marked complete. After its commit is pushed, M3 starts with WSL
  `clip.exe` process handling and tests. Clipboard inspection and end-to-end
  SSH/terminal checks remain.

### 2026-11-11 — M3 WSL and SSH checkpoint

- Implemented the WSL backend by sending UTF-16LE text to built-in `clip.exe`.
  Invalid UTF-8 and embedded NUL are rejected; process startup, input, wait, and
  nonzero-exit errors are surfaced.
- Added encoding and process-failure unit tests plus a binary-level CLI test
  using a stub `clip.exe`. The WSL host clipboard read-back confirmed Unicode
  and no trailing newline; the same WSL copy path worked inside a tmux pane.
- An SSH-signalled pseudo-terminal emitted the expected OSC 52 sequence. This
  was followed by an actual SSH run on `hop`; the remote process selected OSC
  52 and its bytes reached the SSH client's allocated PTY exactly. The tool
  harness captures the bytes, so it cannot confirm the user's local terminal
  processes OSC 52 into the Windows clipboard.
- Changed: `src/wsl.rs`, `src/lib.rs`, `src/main.rs`, `tests/wsl_cli.rs`, and
  this tracker.
- M3 remains blocked until OSC 52 is checked in the actual local terminal and
  the Windows clipboard read back. The temporary binary was removed from `hop`.
  No M4 work has started.

## New-session kickoff

Use this prompt in a new Copilot CLI conversation:

> Read `AGENTS.md`, `docs/project/IMPLEMENTATION_PLAN.md`, and
> `docs/project/PROGRESS.md`. Continue the exact Next action from the progress
> tracker. Follow the progress protocol, keep the default Rust build free of
> third-party crates, run focused validation, and update `PROGRESS.md` before
> ending even if the task remains incomplete.
