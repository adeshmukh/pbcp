pub fn wrap(sequence: &[u8]) -> Vec<u8> {
    let mut wrapped = Vec::with_capacity(sequence.len() + 4);
    wrapped.extend_from_slice(b"\x1bP");
    wrapped.extend_from_slice(sequence);
    wrapped.extend_from_slice(b"\x1b\\");
    wrapped
}

#[cfg(test)]
mod tests {
    use super::wrap;

    #[test]
    fn wraps_osc52_for_screen_passthrough() {
        assert_eq!(
            wrap(b"\x1b]52;c;aGVsbG8=\x07"),
            b"\x1bP\x1b]52;c;aGVsbG8=\x07\x1b\\"
        );
    }

    #[test]
    fn passes_osc_sequence_without_changing_its_bytes() {
        assert_eq!(
            wrap(b"\x1b]52;c;AA==\x07"),
            b"\x1bP\x1b]52;c;AA==\x07\x1b\\"
        );
    }
}
