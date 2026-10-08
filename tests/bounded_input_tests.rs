//! Synthetic readers/writers checking actual IO consumption and failure contracts.
use jsonrepair_rs::{
    jsonrepair_reader_to_writer_with_options, JsonRepairStreamError, RepairOptions,
};
use std::error::Error;
use std::io::{self, Cursor, Read, Write};

#[derive(Default)]
struct Writer {
    calls: usize,
    bytes: Vec<u8>,
}
impl Write for Writer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.calls += 1;
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        self.calls += 1;
        Ok(())
    }
}
struct Chunks<'a> {
    cursor: Cursor<&'a [u8]>,
    size: usize,
}
impl Read for Chunks<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let len = buffer.len().min(self.size);
        self.cursor.read(&mut buffer[..len])
    }
}
#[test]
fn exact_limit_and_limit_plus_one_with_utf8_and_chunks() {
    let input = "{text:'中文🦀'}";
    for chunk in 1..=5 {
        let mut source = Chunks {
            cursor: Cursor::new(input.as_bytes()),
            size: chunk,
        };
        let mut writer = Writer::default();
        jsonrepair_reader_to_writer_with_options(
            &mut source,
            &mut writer,
            RepairOptions::new().with_max_input_bytes(input.len()),
        )
        .unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&writer.bytes).unwrap()["text"],
            "中文🦀"
        );
        let mut source = Chunks {
            cursor: Cursor::new(input.as_bytes()),
            size: chunk,
        };
        let mut writer = Writer::default();
        let limit = 5;
        let error = jsonrepair_reader_to_writer_with_options(
            &mut source,
            &mut writer,
            RepairOptions::new().with_max_input_bytes(limit),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            JsonRepairStreamError::InputTooLarge { limit: 5 }
        ));
        assert_eq!(source.cursor.position(), 6);
        assert_eq!(writer.calls, 0);
        assert!(error.source().is_none());
    }
}
#[test]
fn reject_repairable_truncated_prefix_and_invalid_utf8_excess() {
    for input in [b"{a:123}".as_slice(), b"0\xffmore".as_slice()] {
        let mut reader = Cursor::new(input);
        let mut writer = Writer::default();
        let err = jsonrepair_reader_to_writer_with_options(
            &mut reader,
            &mut writer,
            RepairOptions::new().with_max_input_bytes(1),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            JsonRepairStreamError::InputTooLarge { limit: 1 }
        ));
        assert_eq!(reader.position(), 2);
        assert_eq!(writer.calls, 0);
    }
}
#[test]
fn zero_and_maximum_limits_do_not_overflow() {
    for (input, limit) in [
        (b"0".as_slice(), 0),
        (b"0".as_slice(), usize::MAX),
        (b"".as_slice(), 0),
    ] {
        let mut reader = Cursor::new(input);
        let mut writer = Writer::default();
        let result = jsonrepair_reader_to_writer_with_options(
            &mut reader,
            &mut writer,
            RepairOptions::new().with_max_input_bytes(limit),
        );
        match (input.is_empty(), limit) {
            (true, _) => assert!(matches!(result, Err(JsonRepairStreamError::Repair(_)))),
            (false, 0) => assert!(matches!(
                result,
                Err(JsonRepairStreamError::InputTooLarge { limit: 0 })
            )),
            _ => assert!(result.is_ok()),
        }
        if result.is_err() {
            assert_eq!(writer.calls, 0);
        }
    }
}
#[test]
fn bounded_read_and_utf8_errors_retain_read_contract() {
    struct Failure;
    impl Read for Failure {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::TimedOut, "fixture"))
        }
    }
    let mut writer = Writer::default();
    let error = jsonrepair_reader_to_writer_with_options(
        Failure,
        &mut writer,
        RepairOptions::new().with_max_input_bytes(2),
    )
    .unwrap_err();
    assert!(
        matches!(error, JsonRepairStreamError::Read(ref e) if e.kind() == io::ErrorKind::TimedOut)
    );
    assert_eq!(writer.calls, 0);
    let error = jsonrepair_reader_to_writer_with_options(
        Cursor::new(b"\xff"),
        &mut writer,
        RepairOptions::new().with_max_input_bytes(1),
    )
    .unwrap_err();
    assert!(
        matches!(error, JsonRepairStreamError::Read(ref e) if e.kind() == io::ErrorKind::InvalidData)
    );
    assert_eq!(writer.calls, 0);
}
#[test]
fn tool_policy_is_shared_by_reader_writer_and_serde() {
    let input = r##"{"content": // code\n/* block */ # heading"}"##;
    let options = RepairOptions::new()
        .with_preserve_comment_markers(true)
        .with_decode_unquoted_escapes(true)
        .with_max_input_bytes(input.len());
    let repaired = jsonrepair_rs::jsonrepair_with_options(input, options).unwrap();
    let mut writer = Writer::default();
    jsonrepair_reader_to_writer_with_options(Cursor::new(input), &mut writer, options).unwrap();
    assert_eq!(writer.bytes, repaired.as_bytes());
    let mut output = Vec::new();
    jsonrepair_rs::jsonrepair_to_writer_with_options(input, &mut output, options).unwrap();
    assert_eq!(output, writer.bytes);
    #[cfg(feature = "serde")]
    {
        let value = jsonrepair_rs::jsonrepair_value_with_options(input, options).unwrap();
        assert_eq!(value["content"], "// code\n/* block */ # heading");
        let error = jsonrepair_rs::jsonrepair_value_with_options(
            input,
            RepairOptions::new()
                .with_preserve_comment_markers(true)
                .with_decode_unquoted_escapes(true)
                .with_max_input_bytes(1),
        )
        .unwrap_err();
        assert!(
            matches!(error, jsonrepair_rs::JsonRepairParseError::Repair(e)
            if e.kind == jsonrepair_rs::JsonRepairErrorKind::InputTooLarge)
        );
    }
}

#[test]
fn bounded_reader_write_failure_can_leave_partial_output() {
    struct Partial(Vec<u8>);
    impl Write for Partial {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.0.is_empty() {
                self.0.push(bytes[0]);
                Ok(1)
            } else {
                Err(io::Error::new(io::ErrorKind::BrokenPipe, "fixture"))
            }
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut writer = Partial(Vec::new());
    let error = jsonrepair_reader_to_writer_with_options(
        Cursor::new("{key:1}"),
        &mut writer,
        RepairOptions::new().with_max_input_bytes(7),
    )
    .unwrap_err();
    assert!(
        matches!(error, JsonRepairStreamError::Write(ref e) if e.kind() == io::ErrorKind::BrokenPipe)
    );
    assert_eq!(writer.0, b"{");
}
