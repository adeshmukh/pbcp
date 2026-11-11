use std::io::{self, Read};

pub fn read_input<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    let mut input = Vec::new();
    reader.read_to_end(&mut input)?;
    Ok(input)
}

#[cfg(test)]
mod tests {
    use super::read_input;
    use std::io::{self, Cursor, Read};

    #[test]
    fn reads_empty_input() {
        assert_eq!(read_input(&mut Cursor::new([])).unwrap(), b"");
    }

    #[test]
    fn preserves_input_bytes() {
        let bytes = [0x00, 0xff, b'a', b'\n'];
        assert_eq!(read_input(&mut Cursor::new(bytes)).unwrap(), bytes);
    }

    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("read failed"))
        }
    }

    #[test]
    fn returns_input_errors() {
        assert_eq!(
            read_input(&mut FailingReader).unwrap_err().to_string(),
            "read failed"
        );
    }
}
