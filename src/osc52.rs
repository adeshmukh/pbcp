use crate::base64;

pub fn generate(input: &[u8]) -> Vec<u8> {
    let encoded = base64::encode(input);
    let mut sequence = Vec::with_capacity(7 + encoded.len());
    sequence.extend_from_slice(b"\x1b]52;c;");
    sequence.extend_from_slice(encoded.as_bytes());
    sequence.push(b'\x07');
    sequence
}

#[cfg(test)]
mod tests {
    use super::generate;

    #[test]
    fn generates_exact_osc52_bytes() {
        assert_eq!(generate(b"hello"), b"\x1b]52;c;aGVsbG8=\x07");
    }

    #[test]
    fn generates_empty_clipboard_sequence() {
        assert_eq!(generate(b""), b"\x1b]52;c;\x07");
    }

    #[test]
    fn preserves_arbitrary_input_bytes_through_encoding() {
        assert_eq!(generate(&[0x00, 0xff]), b"\x1b]52;c;AP8=\x07");
    }
}
