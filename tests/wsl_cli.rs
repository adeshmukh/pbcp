#![cfg(unix)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

static NEXT_TEMP_ID: AtomicUsize = AtomicUsize::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::current_dir()
            .unwrap()
            .join("target")
            .join(format!("pbcp-wsl-cli-{}-{id}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn command(&self) -> Command {
        let clip = self.0.join("clip.exe");
        fs::write(&clip, b"#!/bin/sh\nexec /bin/cat > \"$CLIP_CAPTURE\"\n").unwrap();
        fs::set_permissions(&clip, fs::Permissions::from_mode(0o755)).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_pbcp"));
        command
            .env_clear()
            .env("PATH", &self.0)
            .env("WSL_INTEROP", "test-presence")
            .env("CLIP_CAPTURE", self.capture())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }

    fn capture(&self) -> PathBuf {
        self.0.join("captured-input")
    }

    fn config_arg(&self, bytes: &[u8]) -> std::ffi::OsString {
        let path = self.0.join("config");
        fs::write(&path, bytes).unwrap();
        config_arg(&path)
    }
}

fn config_arg(path: &std::path::Path) -> std::ffi::OsString {
    let mut argument = std::ffi::OsString::from("--config=");
    argument.push(path);
    argument
}

fn copy_output(command: &mut Command, input: &[u8]) -> Output {
    let mut child = command.spawn().unwrap();
    let result = child.stdin.take().unwrap().write_all(input);
    if let Err(error) = result {
        assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
    }
    child.wait_with_output().unwrap()
}

fn without_stdin_read(mut child: Child) -> Output {
    let open_stdin = child.stdin.take().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while child.try_wait().unwrap().is_none() {
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("command waited for stdin instead of resolving settings");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    drop(open_stdin);
    child.wait_with_output().unwrap()
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn cli_automatically_routes_wsl_input_to_clip_process() {
    let temp_dir = TempDir::new();
    let captured_input = temp_dir.0.join("captured-input");
    let clip = temp_dir.0.join("clip.exe");
    fs::write(&clip, b"#!/bin/sh\nexec /bin/cat > \"$CLIP_CAPTURE\"\n").unwrap();
    fs::set_permissions(&clip, fs::Permissions::from_mode(0o755)).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_pbcp"))
        .arg("--no-config")
        .env_clear()
        .env("PATH", &temp_dir.0)
        .env("WSL_INTEROP", "test-presence")
        .env("CLIP_CAPTURE", &captured_input)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all("clipboard ☃".as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    assert_eq!(
        fs::read(captured_input).unwrap(),
        "clipboard ☃"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>()
    );
    assert!(!String::from_utf8_lossy(&output.stderr).contains("test-presence"));
}

#[test]
fn diagnose_resolves_values_sources_and_warnings_without_reading_or_copying() {
    let directory = TempDir::new();
    let argument = directory.config_arg(
        b"max_bytes=9\nbackend=osc52\nbackend=auto\nfuture=private-configuration-value",
    );
    let output = without_stdin_read(
        directory
            .command()
            .args([argument])
            .arg("--diagnose")
            .arg("--max-bytes=4")
            .spawn()
            .unwrap(),
    );
    assert!(output.status.success());
    let report = String::from_utf8(output.stdout).unwrap();
    let warnings = String::from_utf8(output.stderr).unwrap();
    assert!(report.contains("Setting backend: auto (source: "));
    assert!(report.contains("config\":3)"));
    assert!(report.contains("Setting max_bytes: 4 (source: CLI)"));
    assert!(report.contains("loaded"));
    assert!(report.contains("skipped (--config replacement)"));
    assert!(report.contains("Selection: automatic"));
    assert!(report.contains("Backend: wsl-clip"));
    assert!(warnings.contains("duplicate key backend; overrides line 2"));
    assert!(warnings.contains("unknown key future"));
    assert!(!format!("{report}{warnings}").contains("private-configuration-value"));
    assert!(!report.contains("test-presence"));
    assert!(!directory.capture().exists());
}

#[test]
fn disabled_discovery_ignores_invalid_xdg_and_reports_builtins() {
    let directory = TempDir::new();
    let output = without_stdin_read(
        directory
            .command()
            .args(["--no-config", "--diagnose"])
            .env("XDG_CONFIG_HOME", "relative")
            .env("HOME", "relative")
            .spawn()
            .unwrap(),
    );
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let report = String::from_utf8(output.stdout).unwrap();
    assert!(report.contains("Setting max_bytes: 65536 (source: built-in)"));
    assert!(report.contains("skipped (--no-config)"));
    assert!(!directory.capture().exists());
}

#[test]
fn explicit_file_and_cli_override_are_order_independent_and_replace_user_file() {
    for cli_first in [false, true] {
        let directory = TempDir::new();
        let user = directory.0.join("pbcp");
        fs::create_dir(&user).unwrap();
        fs::write(user.join("config"), b"backend=invalid").unwrap();
        let argument = directory.config_arg(b"backend=osc52\nmax_bytes=1");
        let mut command = directory.command();
        command.env("XDG_CONFIG_HOME", &directory.0);
        if cli_first {
            command.current_dir(&directory.0);
            command
                .args(["--backend=auto", "--max-bytes=4"])
                .arg("--config=config");
        } else {
            command
                .arg(argument)
                .args(["--max-bytes=4", "--backend=auto"]);
        }
        let output = copy_output(&mut command, b"four");
        assert!(output.status.success(), "{:?}", output.stderr);
        assert!(output.stderr.is_empty());
        assert_eq!(fs::read(directory.capture()).unwrap(), b"f\0o\0u\0r\0");
    }
}

#[test]
fn invalid_or_unreadable_explicit_files_fail_before_stdin_or_clipboard() {
    for bytes in [
        b"max_bytes=0\nmax_bytes=4".as_slice(),
        b"backend=invalid\nbackend=auto",
        b"[section]",
        b"max_bytes=4 # inline",
        &[0xff],
    ] {
        let directory = TempDir::new();
        let argument = directory.config_arg(bytes);
        let output = without_stdin_read(
            directory
                .command()
                .arg(argument)
                .args(["--max-bytes=4", "--backend=auto"])
                .spawn()
                .unwrap(),
        );
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!directory.capture().exists());
    }
    for missing in [false, true] {
        let directory = TempDir::new();
        let path = if missing {
            directory.0.join("absent")
        } else {
            directory.0.clone()
        };
        let output =
            without_stdin_read(directory.command().arg(config_arg(&path)).spawn().unwrap());
        assert!(!output.status.success());
        assert!(String::from_utf8(output.stderr)
            .unwrap()
            .contains("cannot load configuration"));
        assert!(!directory.capture().exists());
    }
}

#[test]
fn default_limit_allows_empty_and_exact_input_and_rejects_one_over_without_copying() {
    for length in [0, 65_536, 65_537] {
        let directory = TempDir::new();
        let output = copy_output(directory.command().arg("--no-config"), &vec![b'x'; length]);
        if length <= 65_536 {
            assert!(output.status.success(), "{:?}", output.stderr);
            assert_eq!(
                fs::read(directory.capture()).unwrap(),
                b"x\0".repeat(length)
            );
        } else {
            assert!(!output.status.success());
            assert!(!directory.capture().exists());
            assert!(String::from_utf8(output.stderr)
                .unwrap()
                .contains("input exceeds max_bytes limit (65536 bytes)"));
        }
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn file_and_cli_limits_share_the_same_exact_boundary() {
    for cli in [false, true] {
        for input in [b"".as_slice(), b"abc", b"abcd"] {
            let directory = TempDir::new();
            let mut command = directory.command();
            if cli {
                command.arg("--no-config").arg("--max-bytes=3");
            } else {
                command.arg(directory.config_arg(b"max_bytes=3"));
            }
            let output = copy_output(&mut command, input);
            assert_eq!(output.status.success(), input.len() <= 3);
            assert_eq!(directory.capture().exists(), input.len() <= 3);
            assert!(output.stdout.is_empty());
        }
    }
}

#[test]
fn cli_errors_and_wsl_text_restrictions_have_no_clipboard_side_effects() {
    for args in [
        vec!["--no-config", "--config=missing"],
        vec!["--no-config", "--max-bytes=0"],
        vec!["--no-config", "--max-bytes=unlimited"],
        vec!["--no-config", "--max-bytes=18446744073709551615"],
        vec!["--no-config", "--max-bytes=1", "--max-bytes=2"],
        vec!["--no-config", "--backend=secret\x1b"],
        vec!["--no-config", "--backend=wsl-clip", "--backend=auto"],
    ] {
        let directory = TempDir::new();
        let output = without_stdin_read(directory.command().args(args).spawn().unwrap());
        assert!(!output.status.success());
        assert!(!output.stderr.contains(&b'\x1b'));
        assert!(!directory.capture().exists());
    }
    for input in [b"\xff".as_slice(), b"a\0b"] {
        let directory = TempDir::new();
        let output = copy_output(directory.command().arg("--no-config"), input);
        assert!(!output.status.success());
        assert!(!directory.capture().exists());
    }
    let directory = TempDir::new();
    let output = without_stdin_read(
        directory
            .command()
            .arg(directory.config_arg(b"backend=wsl-clip"))
            .env_remove("WSL_INTEROP")
            .spawn()
            .unwrap(),
    );
    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("only available in WSL"));
    assert!(!directory.capture().exists());
}

#[test]
fn read_failure_does_not_launch_clipboard_process() {
    let directory = TempDir::new();
    let output = directory
        .command()
        .arg("--no-config")
        .stdin(fs::File::open(&directory.0).unwrap())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("cannot read standard input"));
    assert!(!directory.capture().exists());
}

#[test]
fn native_control_bearing_explicit_path_is_loaded_and_safely_reported() {
    use std::os::unix::ffi::OsStringExt;
    let directory = TempDir::new();
    let path = directory
        .0
        .join(std::ffi::OsString::from_vec(b"config-\xff\n\x1b".to_vec()));
    fs::write(&path, b"future=private\nmax_bytes=4").unwrap();
    let output = without_stdin_read(
        directory
            .command()
            .arg(config_arg(&path))
            .arg("--diagnose")
            .spawn()
            .unwrap(),
    );
    assert!(output.status.success());
    let report = String::from_utf8(output.stdout).unwrap();
    let warnings = String::from_utf8(output.stderr).unwrap();
    assert!(report.contains("\\xFF\\n\\u{1b}"));
    assert!(warnings.contains("\\xFF\\n\\u{1b}"));
    assert!(!report.contains('\x1b'));
    assert!(!warnings.contains('\x1b'));
    assert!(!warnings.contains("private"));
    assert!(!directory.capture().exists());
}
