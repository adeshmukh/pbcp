#![cfg(unix)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_TEMP_ID: AtomicUsize = AtomicUsize::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("pbcp-wsl-cli-{}-{id}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
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
