use std::io::{self, Write};
use std::process::{Command, Stdio};

pub fn copy(input: &[u8]) -> io::Result<()> {
    let mut command = Command::new("clip.exe");
    copy_with_command(&mut command, input)
}

fn copy_with_command(command: &mut Command, input: &[u8]) -> io::Result<()> {
    let encoded_input = encode_text(input)?;
    let mut child = command
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|error| io::Error::new(error.kind(), format!("cannot start clip.exe: {error}")))?;

    let write_result = match child.stdin.take() {
        Some(mut stdin) => stdin.write_all(&encoded_input).map_err(|error| {
            io::Error::new(error.kind(), format!("cannot write to clip.exe: {error}"))
        }),
        None => Err(io::Error::other(
            "clip.exe was started without a writable standard input",
        )),
    };
    let wait_result = child.wait();

    if let Err(write_error) = write_result {
        return match wait_result {
            Ok(_) => Err(write_error),
            Err(wait_error) => Err(io::Error::new(
                write_error.kind(),
                format!("{write_error}; also failed waiting for clip.exe: {wait_error}"),
            )),
        };
    }

    let status = wait_result.map_err(|error| {
        io::Error::new(error.kind(), format!("cannot wait for clip.exe: {error}"))
    })?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "clip.exe exited unsuccessfully: {status}"
        )));
    }

    Ok(())
}

fn encode_text(input: &[u8]) -> io::Result<Vec<u8>> {
    let text = std::str::from_utf8(input).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("WSL clipboard input must be valid UTF-8: {error}"),
        )
    })?;
    if text.contains('\0') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "WSL clipboard input cannot contain NUL bytes",
        ));
    }

    let mut encoded = Vec::new();
    for code_unit in text.encode_utf16() {
        encoded.extend_from_slice(&code_unit.to_le_bytes());
    }
    Ok(encoded)
}

#[cfg(all(test, unix))]
mod tests {
    use super::{copy_with_command, encode_text};
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_TEMP_ID: AtomicUsize = AtomicUsize::new(0);

    struct TempFile(PathBuf);

    impl TempFile {
        fn new() -> Self {
            let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            Self(std::env::temp_dir().join(format!("pbcp-clip-{}-{id}", std::process::id())))
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    #[test]
    fn forwards_unicode_and_input_without_trailing_newline() {
        let output = TempFile::new();
        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg("cat > \"$1\"")
            .arg("pbcp-clip-test")
            .arg(&output.0);

        copy_with_command(&mut command, "clipboard ☃".as_bytes()).unwrap();

        assert_eq!(
            fs::read(&output.0).unwrap(),
            "clipboard ☃"
                .encode_utf16()
                .flat_map(u16::to_le_bytes)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn forwards_empty_input() {
        let output = TempFile::new();
        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg("cat > \"$1\"")
            .arg("pbcp-clip-test")
            .arg(&output.0);

        copy_with_command(&mut command, b"").unwrap();

        assert_eq!(fs::read(&output.0).unwrap(), b"");
    }

    #[test]
    fn encodes_unicode_as_utf16_little_endian_without_bom_or_newline() {
        assert_eq!(
            encode_text("clipboard ☃".as_bytes()).unwrap(),
            b"c\0l\0i\0p\0b\0o\0a\0r\0d\0 \0\x03\x26"
        );
    }

    #[test]
    fn encodes_non_bmp_characters_as_utf16_surrogate_pairs() {
        assert_eq!(encode_text("🚀".as_bytes()).unwrap(), b"\x3d\xd8\x80\xde");
    }

    #[test]
    fn rejects_invalid_utf8_and_nul_input() {
        assert!(encode_text(&[0xff])
            .unwrap_err()
            .to_string()
            .starts_with("WSL clipboard input must be valid UTF-8:"));
        assert_eq!(
            encode_text(b"before\0after").unwrap_err().to_string(),
            "WSL clipboard input cannot contain NUL bytes"
        );
    }

    #[test]
    fn reports_process_start_failure() {
        let mut command = Command::new("/pbcp/nonexistent/clip.exe");
        assert!(copy_with_command(&mut command, b"text")
            .unwrap_err()
            .to_string()
            .starts_with("cannot start clip.exe:"));
    }

    #[test]
    fn reports_nonzero_process_exit() {
        let mut command = Command::new("sh");
        command.arg("-c").arg("exit 7");

        let error = copy_with_command(&mut command, b"text").unwrap_err();
        assert!(error
            .to_string()
            .starts_with("clip.exe exited unsuccessfully:"));
        assert_eq!(error.raw_os_error(), None);
    }
}
