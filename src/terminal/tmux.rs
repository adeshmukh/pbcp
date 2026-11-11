pub fn wrap(sequence: &[u8]) -> Vec<u8> {
    let mut wrapped = Vec::with_capacity(sequence.len() + 10);
    wrapped.extend_from_slice(b"\x1bPtmux;\x1b");
    for &byte in sequence {
        wrapped.push(byte);
        if byte == b'\x1b' {
            wrapped.push(byte);
        }
    }
    wrapped.extend_from_slice(b"\x1b\\");
    wrapped
}

#[cfg(test)]
mod tests {
    use super::wrap;

    #[test]
    fn wraps_osc52_for_tmux_passthrough() {
        assert_eq!(
            wrap(b"\x1b]52;c;aGVsbG8=\x07"),
            b"\x1bPtmux;\x1b\x1b\x1b]52;c;aGVsbG8=\x07\x1b\\"
        );
    }

    #[test]
    fn doubles_each_escape_byte_in_payload() {
        assert_eq!(
            wrap(b"before\x1bmiddle\x1bend"),
            b"\x1bPtmux;\x1bbefore\x1b\x1bmiddle\x1b\x1bend\x1b\\"
        );
    }

    #[test]
    fn wraps_empty_payload() {
        assert_eq!(wrap(b""), b"\x1bPtmux;\x1b\x1b\\");
    }
}
