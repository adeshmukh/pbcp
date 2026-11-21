# pbcp project progress

Last updated: 2026-11-21

This file is the canonical handoff between implementation sessions. Keep it
short, factual, and current. `AGENTS.md` defines the required update protocol.

## Current state

**Phase:** M0 complete; M1 complete; M2 complete; M3 complete; M4 in progress.

**Next action:** Review PR #1 for merge to enable the first automatic GitHub
Release on `main`.

**Blockers:** None.

## Milestone status

| Milestone | Status | Notes |
|---|---|---|
| M0 Project contract | Done | Agent guide, implementation plan, and tracker created |
| M1 Tested core | Done | Zero-dependency crate, CLI parsing, stdin, Base64, and OSC 52 core |
| M2 Terminal transports | Done | CLI wiring, direct output, `tmux` and `screen` passthrough, selection, and presence-only `--diagnose` |
| M3 WSL and SSH vertical slice | Done | WSL `clip.exe` validated directly and from tmux; user verified remote `hop` SSH OSC 52 delivery into the Windows clipboard |
| M4 Product hardening | In progress | Configuration, payload limits, and CI/release setup implemented; binary policy and user-facing README/installer remain |
| M5 macOS | Pending | Requires a Mac or hosted macOS runner for final validation |
| M6 Local graphical Linux | Pending | Wayland and X11 lifecycle work |
| M7 Native Windows/distribution | Pending | Lowest initial priority |

Use only these status values: `Pending`, `In progress`, `Blocked`, `Done`.

## Active task

GitHub CI and release setup: Done on `ci/github-releases`, PR #1. Native Linux
x86-64/ARM64 GNU and musl checks passed, and the combined release bundle was
downloaded and verified. The main-only publisher is mock-tested but has not
published a release; merging the PR enables it. M4 is not complete.
The user explicitly selected this independent M4 work while configuration work
proceeds separately. Only this worktree's tracker is changed; binary-input policy
and the `curl | sh` installer remain pending.

Configuration and payload policy specification: Done.
Implement the pure configuration parser and per-key merge logic with tests:
Done.
File discovery/loading, CLI integration, diagnostics/provenance, and bounded
input enforcement: Done. All required checks passed (58 unit tests and 10 CLI
integration tests). This completes the configuration specification, not M4.

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
- CI uses native Linux x86-64/ARM64 runners and static musl binary artifacts.
  Successful processed main pushes publish GitHub Releases, starting at 0.0.1
  and incrementing the patch. PR/manual runs only upload preview artifacts.
  Serialized runs may coalesce rapid pending pushes. Release notes are generated
  with an explicit platform/validation template; see `docs/releases/README.md`.
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
- Configuration uses dependency-free `key=value` files, layered built-in,
  system, user, then CLI. `--config` replaces automatic discovery and
  `--no-config` disables file loading; details are in `CONFIGURATION.md`.
- Unknown file keys warn and are ignored; duplicate keys within a file warn
  and use the later value. Invalid recognized values always fail, including
  values shadowed by later sources.
- The default payload limit is 65,536 raw-input bytes across backends.
  Overrides must be positive and finite; oversize input fails without copying
  or truncation. Incremental stdin reads and checked expansion arithmetic enforce
  this policy; large configured limits do not eagerly allocate input storage.
- The representability cap is `floor((isize::MAX - 26) / 8) * 3` bytes on the
  compiled architecture, conservatively covering UTF-16 and OSC 52/mux expansion.
  Available memory and receiving-terminal limits remain separate constraints.
- Automatic Unix discovery requires an absolute non-empty XDG root, or an
  absolute HOME for its fallback. Unusable HOME skips user discovery with a
  warning; explicit-file and disabled modes bypass discovery validation.
- Unix configuration paths retain native bytes; diagnostics escape identifiers
  and never echo raw file values or lines.

## Open decisions

- Policy for NUL-containing or otherwise binary input.
- Exit codes and CLI options beyond those specified in `CONFIGURATION.md`.
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
  PTY over SSH. At that point, the actual local terminal's OSC 52 handling was
  still unverified because the tool session captured output.
- M3 manual validation completed by the user: copied the test binary to `hop`,
  ran it over `ssh -tt` from the actual WSL terminal, and confirmed
  `Get-Clipboard -Raw` returned `pbcp SSH validation: snowman ☃`.
- M4 configuration completion: `cargo fmt --all -- --check`,
  `cargo test --all-targets` (58 unit tests and 10 CLI integration tests passed),
  `cargo clippy --all-targets -- -D warnings`, `cargo tree --edges normal`
  (only local `pbcp`, no third-party dependencies), and `git diff --check`
  passed. Formatting was applied after the first final check found formatting
  differences. Pure discovery tests inject file I/O; hermetic CLI tests use
  repository-local scratch directories and stub `clip.exe`, with live stdin
  pipes to verify diagnose/config failures do not read input. Permission-denied
  loading is simulated; directory/missing/invalid-UTF-8 failures are also tested.
  No real system/user configuration, clipboard, terminal, or new platform
  behavior was inspected or manually validated.

- CI branch: local formatting, all 68 Rust tests, Clippy, dependency-tree,
  nine release-planning/publication tests, Bash syntax, actionlint, and whitespace
  checks passed. The actual Git-tag planner returned `version=0.0.1`.
- Hosted run 44940735386 passed all GNU/musl checks on native Linux x86-64 and
  ARM64, static-link/diagnostic checks, artifact upload/download, and combined
  checksum/provenance verification with current Node 24 Actions. Publishing
  was skipped on the PR. The downloaded combined bundle's checksums matched.
  An earlier run's x86-64 binary also executed `--diagnose` on this host.
  No real clipboard functionality or main-push publication was newly validated.

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
- At the checkpoint, M3 was blocked on actual local-terminal verification.
- The user then confirmed the remote SSH test updated the Windows clipboard
  from the actual local terminal. M3 is complete; the temporary binary was
  removed from `hop`.
- No M4 implementation has started. The next task is to decide and test the
  maximum input payload and override policy.

### 2026-11-21 — M4 configuration specification

- Documented the default payload limit, file format and discovery, per-key
  precedence, explicit CLI controls, diagnostics, and warning/error semantics.
- Accepted unknown-key and duplicate-key warnings for compatibility and
  concatenation; invalid recognized values fail even when later overridden.
- Changed: `docs/project/CONFIGURATION.md`,
  `docs/project/IMPLEMENTATION_PLAN.md`, and this tracker.
- Documentation-only work; `git diff --check` passed. No Rust tests or platform
  checks run. Implementation and boundary tests remain pending. No new blockers.
- Next action: implement the pure configuration parser and per-key merge logic
  with focused tests.

### 2026-11-21 — Complete configuration implementation

- Implemented the pure parser/merge first and passed its six focused tests,
  then completed discovery/loading, per-key CLI overrides, provenance and source
  diagnostics, and shared bounded stdin enforcement without dependencies.
- Added invalid-shadowed-value, warning, layering, path, file-I/O, CLI-order,
  default/override boundary, no-input/no-copy, and WSL restriction checks.
  Preserved selection policy and exact protocol bytes; corrected OSC 52 and
  tmux capacity sizing and sized UTF-16 storage from actual input to avoid
  allocation-growth arithmetic exceeding the validated range.
- Changed: `src/{cli,config,diagnose,input,lib,main,osc52,wsl}.rs`,
  `src/terminal/tmux.rs`, `tests/wsl_cli.rs`, and
  `docs/project/{CONFIGURATION,IMPLEMENTATION_PLAN,PROGRESS}.md`.
- Required formatting, all 68 tests, Clippy, dependency-tree, and whitespace
  checks passed. Test scratch paths were cleaned; no real clipboard operations
  or new platform validation occurred.
- Configuration specification: Done. M4: In progress. No new blockers.
  Next action: decide and test the OSC 52 NUL/binary-input policy.

### 2026-11-21 — Isolated CI and release implementation

- Created `.worktrees/github-ci` on `ci/github-releases`, based on the completed
  configuration commit. The primary worktree and application code are untouched.
- Added native Linux x86-64/ARM64 checks, musl test/build/package jobs, read-only
  PR previews, and a main-only GitHub Release publisher with patch allocation,
  generated notes, SHA-256 checksums, and draft/retry safeguards.
- Changed: `.github/workflows/ci.yml`, `.github/scripts/{release.py,
  test_release.py,publish.sh,test_publish.py}`, `Cargo.toml`, `Cargo.lock`,
  `docs/releases/{README,RELEASE_NOTES}.md`, and this tracker.
- Local Rust formatting, all 68 existing tests, Clippy, dependency-tree, nine
  version/stamping/publication tests, Bash syntax, actionlint, and whitespace
  checks passed. GitHub's first-release notes API succeeded without creating a
  release. Hosted CI/artifact validation remains pending. No release or
  additional real clipboard validation has occurred.
- GitHub authentication and Actions access verified. The SSH Git remote was
  unavailable; authenticated HTTPS fetch succeeded without changing that remote.
- CI/release setup: In progress. M4 remains In progress. Next action: validate
  hosted PR CI and its downloadable binary artifacts.
- Opened PR #1. Initial hosted run 44940458122 passed GNU/musl tests and builds on
  both native Linux architectures; publishing was skipped as intended. Downloaded
  both archives, verified SHA-256/provenance/static architectures, and ran the
  x86-64 artifact's diagnostic path locally. ARM64 execution was hosted, not local.
- The initial run reported Node 20 deprecation warnings. Updated to current pinned
  Node 24 Actions and added read-only combined release-bundle verification on PRs;
  hosted run 44940735386 passed without those warnings. Downloaded and verified
  its combined bundle (both checksums and embedded commit/version/target).
- CI/release setup: Done. PR #1 is unmerged; no releases were created. M4 remains
  In progress. Main-only publishing was tested using mock `gh` calls, not an
  actual release. Next action: review PR #1 for merge and first-release rollout.

## New-session kickoff

Use this prompt in a new Copilot CLI conversation:

> Read `AGENTS.md`, `docs/project/IMPLEMENTATION_PLAN.md`, and
> `docs/project/PROGRESS.md`. Continue the exact Next action from the progress
> tracker. Follow the progress protocol, keep the default Rust build free of
> third-party crates, run focused validation, and update `PROGRESS.md` before
> ending even if the task remains incomplete.
