#![no_main]

use std::io::{Cursor, Read};

use libfuzzer_sys::fuzz_target;

struct Chunked<R> {
    inner: R,
    chunk_size: usize,
}

impl<R: Read> Read for Chunked<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = buf.len().min(self.chunk_size);
        self.inner.read(&mut buf[..len])
    }
}

fuzz_target!(|data: &[u8]| {
    let Some((&size, bytes)) = data.split_first() else {
        return;
    };
    let input = String::from_utf8_lossy(&bytes[..bytes.len().min(4096)]);
    let expected = jsonrepair_rs::jsonrepair(&input);
    let reader = Chunked {
        inner: Cursor::new(input.as_bytes()),
        chunk_size: usize::from(size % 32 + 1),
    };
    let mut output = Vec::new();
    let actual = jsonrepair_rs::jsonrepair_reader_to_writer(reader, &mut output);

    match (expected, actual) {
        (Ok(repaired), Ok(())) => {
            assert_eq!(output, repaired.as_bytes());
            serde_json::from_slice::<serde_json::Value>(&output)
                .expect("successful stream repairs must be valid JSON");
        }
        (Err(_), Err(jsonrepair_rs::JsonRepairStreamError::Repair(_))) => {
            assert!(output.is_empty(), "repair failure must not write partial output");
        }
        (expected, actual) => panic!("string/stream result mismatch: {expected:?} vs {actual:?}"),
    }
});
