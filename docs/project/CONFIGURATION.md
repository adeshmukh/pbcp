# Configuration specification

Status: implemented and validated for the initial Unix/WSL configuration scope.

## Sources and precedence

Resolve settings per key, in this order (later sources override earlier ones):

1. Built-in defaults.
2. Optional system file: `/etc/pbcp/config`.
3. Optional user file: `$XDG_CONFIG_HOME/pbcp/config`, or
   `~/.config/pbcp/config` when `XDG_CONFIG_HOME` is unset or empty.
4. Individual CLI options.

These discovery paths apply to the initial Unix/WSL implementation. A non-empty
`XDG_CONFIG_HOME` must be absolute when automatic discovery applies. Missing,
empty, or relative `HOME` cannot supply the fallback user directory: skip user
discovery and warn; system configuration and CLI options still apply.
Native Windows discovery is deferred to its platform milestone.

System settings are overridable defaults, not enforced administrator policy.
Files are never required or created automatically.

- `--config=/path/to/config` replaces both automatically discovered files with
  one required file; built-in defaults and individual CLI overrides still apply.
- `--no-config` disables all file discovery and loading, but not CLI overrides.
- `--config` and `--no-config` are mutually exclusive.
- Relative explicit paths resolve against the current working directory.
  Unix paths retain native bytes, including non-UTF-8 filenames; diagnostics
  escape these identifiers.
- CLI order does not change precedence. Repeated value-bearing CLI options are
  errors, unlike duplicate file keys.
- No includes, environment-variable setting overrides, or per-backend limits.

## File format

Use a small UTF-8 `key=value` format, parsed without third-party Cargo crates;
this is not JSON, TOML, or a general INI format.

```ini
# Defaults may be overridden by a later file or CLI option.
max_bytes=65536
backend=auto
```

Accept LF and CRLF, blank lines, and whole-line comments beginning with `#`
after optional whitespace. Trim surrounding ASCII whitespace from keys and
values; split assignments at the first `=`. Keys are case-sensitive,
non-empty ASCII identifiers containing letters, digits, and underscores.
Values must be non-empty. Sections, quoting, escaping, interpolation, and
inline comments are not supported.

| Key | Built-in default | CLI override | Accepted values |
|---|---|---|---|
| `max_bytes` | `65536` | `--max-bytes=N` | Positive decimal integer bytes, without signs or unit suffixes |
| `backend` | `auto` | `--backend=NAME` | `auto`, `osc52`, `wsl-clip` |

An explicit `auto` resets inherited backend selection to automatic detection.
Backend selection retains the existing environment checks and explicit errors;
configuration does not make an unavailable backend usable. `--diagnose` is an
invocation action, not a persistent setting.

## Warnings and failures

- Unknown syntactically valid keys: warn and ignore, allowing older binaries
  to read files containing newer settings. Do not validate unknown values
  beyond the assignment syntax.
- Duplicate keys within one file: warn and choose the later value. This permits
  concatenated configuration fragments. Overrides between sources are normal
  and do not warn.
- Invalid recognized values: fail, even if a later assignment, file, or CLI
  option would override them. Every recognized assignment must be valid.
- Malformed syntax, invalid UTF-8, and unreadable existing files: fail.
- Missing automatically discovered files: silently skip. Missing explicitly
  requested files: fail.

Warnings go to stderr and do not by themselves cause a nonzero exit. Identify
the file, line, and key; duplicate warnings identify the earlier line as well.
Do not echo raw values or entire lines. Escape control characters in diagnostic
identifiers so configuration cannot inject terminal control sequences.

Parse and validate every selected file before copying. Never fall back to
defaults after a file error. Unknown-key warnings provide visibility for typos;
they do not guarantee that a setting from a newer release takes effect.

## Payload safety and diagnostics

The default is 64 KiB (65,536 bytes) of raw stdin, before Base64 or UTF-16
encoding, shared by all backends. Empty input and input exactly at the limit
are allowed. Reject larger input before clipboard side effects; never truncate.
Read incrementally with bounded allocation rather than reading everything and
checking afterward.

Overrides must remain positive and finite; `0` and `unlimited` are invalid.
Reject limits outside the implementation's safely representable range and
check size arithmetic for encoding and transport expansion. A large configured
limit must not trigger eager allocation of that entire amount.

The implementation conservatively caps overrides at
`floor((isize::MAX - 26) / 8) * 3` raw bytes for the compiled architecture.
Checked arithmetic accounts for UTF-16LE, padded Base64, the OSC 52 envelope,
and worst-case escape doubling plus multiplexer framing. This is an arithmetic
bound, not a promise that such a large payload can fit in available memory.
The stdin reader uses at most 8 KiB per read, grows storage only as bytes arrive,
and probes for one extra byte at the boundary before any clipboard operation.

The default expands to approximately 87 KiB of Base64. This is a conservative
starting point, not a guarantee of terminal support: raising the limit does not
raise the receiving terminal's clipboard limit. Existing WSL text restrictions
remain unchanged; the broader binary-input policy is a separate M4 decision.

`--diagnose` resolves the same settings without reading stdin or copying. Report
effective settings and provenance (built-in, file path and line, or CLI), plus
which configuration files were loaded or skipped. Never display clipboard data.

## Required implementation checks

Add focused tests for per-key precedence, omitted keys, explicit-file replacement,
disabled discovery, and CLI conflicts. Cover unknown and duplicate warnings,
invalid shadowed values, malformed and unreadable files, and missing-file rules.
Test empty input, the exact payload boundary, one byte over, invalid limits,
and oversize rejection without clipboard output.
