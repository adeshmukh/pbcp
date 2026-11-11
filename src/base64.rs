const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn encode(input: &[u8]) -> String {
    let mut output = String::new();

    for chunk in input.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied();
        let third = chunk.get(2).copied();

        output.push(ALPHABET[(first >> 2) as usize] as char);
        output
            .push(ALPHABET[(((first & 0b11) << 4) | (second.unwrap_or(0) >> 4)) as usize] as char);

        match (second, third) {
            (Some(second), Some(third)) => {
                output.push(ALPHABET[(((second & 0b1111) << 2) | (third >> 6)) as usize] as char);
                output.push(ALPHABET[(third & 0b111111) as usize] as char);
            }
            (Some(second), None) => {
                output.push(ALPHABET[((second & 0b1111) << 2) as usize] as char);
                output.push('=');
            }
            (None, _) => {
                output.push('=');
                output.push('=');
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::encode;

    #[test]
    fn encodes_empty_input() {
        assert_eq!(encode(b""), "");
    }

    #[test]
    fn encodes_inputs_with_each_padding_length() {
        assert_eq!(encode(b"f"), "Zg==");
        assert_eq!(encode(b"fo"), "Zm8=");
        assert_eq!(encode(b"foo"), "Zm9v");
    }

    #[test]
    fn encodes_arbitrary_bytes() {
        assert_eq!(encode(&[0x00, 0xff, 0x10, 0x80]), "AP8QgA==");
    }

    #[test]
    fn encodes_unicode_as_utf8_bytes() {
        assert_eq!(encode("clipboard 🐙".as_bytes()), "Y2xpcGJvYXJkIPCfkJk=");
    }
}
