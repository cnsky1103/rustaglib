/// Searches the ByteVector for `pattern` starting at `offset` and returns
/// the offset.  Returns None if the pattern was not found.  If `byteAlign` is
/// specified the pattern will only be matched if it starts on `byte` divisible
/// by `byteAlign` (starting from `offset`).
/// `offset` by default is 0, and `byte_align` by default is 1
pub(crate) fn byte_vec_find(
    bytes: &Vec<u8>,
    pattern: &Vec<u8>,
    offset: usize,
    byte_align: usize,
) -> Option<usize> {
    let data_size = bytes.len();
    let pattern_size = pattern.len();

    if pattern_size == 0 || offset + pattern_size > data_size {
        return None;
    }

    // n % 0 is invalid
    if byte_align == 0 {
        return None;
    }

    // simple, trivial algorithm is enough
    let mut iter_start = offset;
    while iter_start < data_size - pattern_size + 1 {
        let mut iter_data = iter_start;
        let mut iter_pattern = 0;

        while bytes[iter_data] == pattern[iter_pattern] {
            iter_data += 1;
            iter_pattern += 1;

            if iter_pattern == pattern_size {
                return Some(iter_start);
            }
        }
        iter_start += byte_align;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_vec_find() {
        let data = vec![
            b'H', b'e', b'l', b'l', b'o', b',', b'w', b'o', b'r', b'l', b'd', b'!',
        ];
        let pattern_1 = vec![b'w', b'o'];
        let pattern_2 = vec![b'x'];

        assert_eq!(byte_vec_find(&data, &pattern_1, 0, 1).unwrap(), 6);
        assert!(byte_vec_find(&data, &pattern_2, 0, 1).is_none());
    }

    #[test]
    fn test_byte_vec_find_offset() {
        let data = vec![
            b'H', b'e', b'l', b'l', b'o', b',', b'w', b'o', b'r', b'l', b'd', b'!',
        ];
        let pattern_1 = vec![b'w', b'o'];

        assert_eq!(byte_vec_find(&data, &pattern_1, 3, 1).unwrap(), 6);
        assert!(byte_vec_find(&data, &pattern_1, 7, 1).is_none());
    }

    #[test]
    fn test_byte_vec_find_byte_align() {
        let data = vec![
            b'H', b'e', b'l', b'l', b'o', b',', b'w', b'o', b'r', b'l', b'd', b'!',
        ];
        let pattern_1 = vec![b'w', b'o'];

        assert_eq!(byte_vec_find(&data, &pattern_1, 0, 2).unwrap(), 6);
        assert!(byte_vec_find(&data, &pattern_1, 0, 4).is_none());
    }
}
