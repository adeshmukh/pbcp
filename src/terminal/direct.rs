use std::io::{self, Write};

pub fn write_to_controlling_terminal(sequence: &[u8]) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::fs::OpenOptions;

        let mut terminal = OpenOptions::new()
            .write(true)
            .open("/dev/tty")
            .map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!("cannot open controlling terminal /dev/tty: {error}"),
                )
            })?;
        write_sequence(&mut terminal, sequence).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("cannot write OSC 52 sequence to controlling terminal: {error}"),
            )
        })
    }

    #[cfg(not(unix))]
    {
        let _ = sequence;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "direct controlling-terminal output is unsupported on this platform",
        ))
    }
}

fn write_sequence(writer: &mut impl Write, sequence: &[u8]) -> io::Result<()> {
    writer.write_all(sequence)
}

#[cfg(test)]
mod tests {
    use super::write_sequence;
    use std::io::{self, Write};

    #[derive(Default)]
    struct ShortWriter {
        bytes: Vec<u8>,
    }

    impl Write for ShortWriter {
        fn write(&mut self, input: &[u8]) -> io::Result<usize> {
            let count = input.len().min(2);
            self.bytes.extend_from_slice(&input[..count]);
            Ok(count)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("write failed"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn writes_all_bytes_after_short_writes() {
        let mut writer = ShortWriter::default();
        write_sequence(&mut writer, b"control sequence").unwrap();
        assert_eq!(writer.bytes, b"control sequence");
    }

    #[test]
    fn returns_write_failures() {
        assert_eq!(
            write_sequence(&mut FailingWriter, b"sequence")
                .unwrap_err()
                .to_string(),
            "write failed"
        );
    }
}
