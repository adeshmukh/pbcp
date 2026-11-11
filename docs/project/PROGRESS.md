# pbcp project progress

Last updated: 2026-11-11

This file is the canonical handoff between implementation sessions. Keep it
short, factual, and current. `AGENTS.md` defines the required update protocol.

## Current state

**Phase:** M0 complete; M1 complete; M2 in progress.

**Next action:** Implement and test deterministic backend selection for explicit
`--backend`, SSH, and WSL signals.

**Blockers:** None. Terminal and WSL behavior has not been manually validated.

## Milestone status

| Milestone | Status | Notes |
|---|---|---|
| M0 Project contract | Done | Agent guide, implementation plan, and tracker created |
| M1 Tested core | Done | Zero-dependency crate, CLI parsing, stdin, Base64, and OSC 52 core |
| M2 Terminal transports | In progress | Direct output, `tmux` passthrough, and environment snapshot; `--diagnose` remains |
| M3 WSL and SSH vertical slice | Pending | Primary user value |
| M4 Product hardening | Pending | Required before first release |
| M5 macOS | Pending | Requires a Mac or hosted macOS runner for final validation |
| M6 Local graphical Linux | Pending | Wayland and X11 lifecycle work |
| M7 Native Windows/distribution | Pending | Lowest initial priority |

Use only these status values: `Pending`, `In progress`, `Blocked`, `Done`.

## Active task

Implement deterministic backend selection for explicit `--backend`, SSH, and
WSL signals, with focused tests.

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
- GPT-6 Luna is preferred for focused implementation, GPT-6 Sol for correctness
  review and integration, and GPT-6 Astra only for unresolved difficult work.

## Open decisions

- Maximum clipboard payload and override behavior.
- Policy for NUL-containing or otherwise binary input.
- Exact CLI spelling and exit codes beyond `--backend` and `--diagnose`.
- Reliable automatic capability heuristics versus explicit OSC 52 selection.
- Whether `screen` support fits the initial release after its transport is
  investigated.

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
- No direct-terminal, tmux-session, WSL, or SSH end-to-end behavior has been
  manually validated.

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

## New-session kickoff

Use this prompt in a new Copilot CLI conversation:

> Read `AGENTS.md`, `docs/project/IMPLEMENTATION_PLAN.md`, and
> `docs/project/PROGRESS.md`. Continue the exact Next action from the progress
> tracker. Follow the progress protocol, keep the default Rust build free of
> third-party crates, run focused validation, and update `PROGRESS.md` before
> ending even if the task remains incomplete.
