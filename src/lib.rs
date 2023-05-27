pub mod serialize;

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use crate::serialize::{write_byte_arr, write_string, read_string};

    #[test]
    fn serialize_bytes() {
        let bytes = [0u8, 1, 3, 5, 7, 11];
        let mut buf = vec![];
        assert!(write_byte_arr(&mut buf, &bytes).is_ok());
    }

    #[test]
    fn serialize_deserialize() {
        let text = "Hello world.";
        let mut buf = vec![];
        assert!(write_string(&mut buf, text).is_ok());

        let mut cursor = Cursor::new(buf.as_slice());
        let new_text = read_string(&mut cursor).unwrap();
        assert_eq!(text, new_text);
    }
}
