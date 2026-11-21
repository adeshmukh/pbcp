use std::io::{self, Read};

pub fn read_input<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    read_input_with_limit(reader, crate::config::DEFAULT_MAX_BYTES)
}

pub fn read_input_with_limit<R: Read>(reader: &mut R, limit: usize) -> io::Result<Vec<u8>> {
    if !(1..=crate::config::MAX_SAFE_BYTES).contains(&limit) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid input byte limit",
        ));
    }
    let mut input = Vec::new();
    let mut buffer = [0; 8192];
    loop {
        let count = buffer.len().min(limit - input.len() + 1);
        let read = match reader.read(&mut buffer[..count]) {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            result => result?,
        };
        if read == 0 {
            return Ok(input);
        }
        if read > limit - input.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("input exceeds max_bytes limit ({limit} bytes); nothing copied"),
            ));
        }
        input
            .try_reserve_exact(read)
            .map_err(|_| io::Error::other("cannot allocate storage for standard input"))?;
        input.extend_from_slice(&buffer[..read]);
    }
}

#[cfg(test)]
mod tests {
    use super::{read_input, read_input_with_limit};
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

    #[test]
    fn permits_empty_and_exact_limit_but_rejects_one_over() {
        assert!(read_input_with_limit(&mut Cursor::new([]), 1)
            .unwrap()
            .is_empty());
        assert_eq!(
            read_input_with_limit(&mut Cursor::new(b"abc"), 3).unwrap(),
            b"abc"
        );
        assert!(read_input_with_limit(&mut Cursor::new(b"abcd"), 3).is_err());
        let bytes = vec![b'a'; crate::config::DEFAULT_MAX_BYTES];
        assert_eq!(read_input(&mut Cursor::new(&bytes)).unwrap(), bytes);
        assert!(read_input(&mut Cursor::new(vec![b'a'; bytes.len() + 1])).is_err());
    }

    #[test]
    fn reads_incrementally_without_allocating_the_configured_limit() {
        struct ChunkedReader {
            calls: usize,
        }
        impl Read for ChunkedReader {
            fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
                assert!(buffer.len() <= 8192);
                self.calls += 1;
                match self.calls {
                    1 => Err(io::ErrorKind::Interrupted.into()),
                    2 => {
                        buffer[0] = b'a';
                        Ok(1)
                    }
                    _ => Ok(0),
                }
            }
        }
        let bytes = read_input_with_limit(
            &mut ChunkedReader { calls: 0 },
            crate::config::MAX_SAFE_BYTES,
        )
        .unwrap();
        assert_eq!(bytes, b"a");
        assert!(bytes.capacity() < 8192);
        assert!(read_input_with_limit(&mut Cursor::new([]), 0).is_err());
    }

    #[test]
    fn stops_reading_at_first_byte_over_limit_and_propagates_late_failures() {
        let mut reader = Cursor::new(vec![b'x'; 100]);
        assert!(read_input_with_limit(&mut reader, 4).is_err());
        assert_eq!(reader.position(), 5);
        struct LateFailure(bool);
        impl Read for LateFailure {
            fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
                if self.0 {
                    return Err(io::Error::other("late read failed"));
                }
                self.0 = true;
                buffer[0] = b'x';
                Ok(1)
            }
        }
        assert_eq!(
            read_input_with_limit(&mut LateFailure(false), 1)
                .unwrap_err()
                .to_string(),
            "late read failed"
        );
    }
}
