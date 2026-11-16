# pbcp contributor guide

Before changing code, read:

1. `docs/project/IMPLEMENTATION_PLAN.md`
2. `docs/project/PROGRESS.md`

The implementation plan is the durable product and architecture contract.
`PROGRESS.md` is the canonical execution state and must always name one concrete
next action.

## Progress protocol

At the start of a work session:

1. Confirm that the current task matches the **Next action** in `PROGRESS.md`.
2. Mark its milestone as in progress before editing.
3. If the next action is blocked or obsolete, update the tracker with the reason
   and select the smallest unblocked action consistent with the plan.

Before ending a work session, even when work is incomplete:

1. Update milestone and task statuses.
2. Record files changed and validation actually run.
3. Record new durable decisions or unresolved blockers.
4. Replace **Next action** with one exact, independently actionable task.
5. Append a short entry to the session log.

Do not record intended tests as completed tests, and do not claim a platform was
validated when it was only compiled or simulated.

## Engineering constraints

- Implement the CLI in Rust.
- Keep the default build free of third-party Cargo dependencies.
- Prefer a single self-contained binary and native protocols or OS facilities.
- System APIs and guaranteed host facilities such as WSL interoperability are
  allowed; optional external clipboard utilities must not be required.
- Prioritize WSL, SSH, direct terminals, and `tmux`. Support GNU `screen` when
  doing so remains a small transport-layer addition.
- Isolate unsafe platform FFI in small platform modules.
- Keep protocol generation pure and byte-testable; keep terminal I/O at the
  boundary.
- Return explicit errors rather than silently selecting a nonfunctional backend.
- Preserve exact input bytes where the selected clipboard protocol supports
  them. Any text-only restriction must be deliberate, tested, and documented.

## Execution strategy

Do not split the initial foundation across worktrees: the CLI contract, backend
interface, detection policy, and OSC 52 transport are tightly coupled. After
that contract is stable, independent platform backends may be developed in
parallel worktrees.

The current model strategy is GPT-6 Luna for focused implementation and tests,
GPT-6 Sol for integration and independent correctness review, and GPT-6 Astra
only for difficult unresolved protocol or FFI problems.
