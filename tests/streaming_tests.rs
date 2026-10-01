use std::io::{self, Cursor, Read, Write};

use jsonrepair_rs::{jsonrepair, jsonrepair_reader_to_writer, JsonRepairStreamError};

#[test]
fn repairs_reader_to_writer() {
    let mut input = Cursor::new("{name: 'Ada', active: True}");
    let mut output = Vec::new();

    jsonrepair_reader_to_writer(&mut input, &mut output).unwrap();

    assert_eq!(
        String::from_utf8(output).unwrap(),
        r#"{"name": "Ada", "active": true}"#
    );
}

#[test]
fn known_wrappers_multiple_arguments_across_reader_chunks() {
    for (input, expected) in [
        (r#"callback({"a":1},2)"#, r#"[{"a":1},2]"#),
        ("callback(1,2,3)", "[1,2,3]"),
        (r#"ObjectId("a","b")"#, r#"["a","b"]"#),
        ("NumberLong(1,2)", "[1,2]"),
        (r#"callback({"a":1},2);"#, r#"[{"a":1},2]"#),
        ("callback(callback(1,2),3)", "[[1,2],3]"),
        (r#"callback(callback({"a":1}),2)"#, r#"[{"a":1},2]"#),
        (r#"callback({"a":1})"#, r#"{"a":1}"#),
        (r#"ObjectId("abc")"#, r#""abc""#),
        (r#"{"x":callback(1,2}"#, r#"{"x":[1,2]}"#),
        ("[callback(1,2]", "[[1,2]]"),
        (r#"{"x":callback(callback(1,2),3}"#, r#"{"x":[[1,2],3]}"#),
        (r#"{"x":ObjectId("a","b"}"#, r#"{"x":["a","b"]}"#),
        (r#"{"x":callback(1}"#, r#"{"x":1}"#),
        ("[callback(1]", "[1]"),
        (r#"{"x":callback(1,"y":2}"#, r#"{"x":1,"y":2}"#),
        ("{x:callback(1,y:2}", r#"{"x":1,"y":2}"#),
        (r#"{"x":callback(1,2,"y":3}"#, r#"{"x":[1,2],"y":3}"#),
        ("{x:callback(1,2,y:3}", r#"{"x":[1,2],"y":3}"#),
        (
            r#"{"x":callback({"a":1},"y":[2,3]}"#,
            r#"{"x":{"a":1},"y":[2,3]}"#,
        ),
        (
            r#"{"x":callback(1, /* key */ 'y' /* colon */ :2}"#,
            r#"{"x":1,  "y"  :2}"#,
        ),
        (r#"{"x":callback(1,"y\"z":2}"#, r#"{"x":1,"y\"z":2}"#),
        (r#"{"x":callback(1,键:2}"#, r#"{"x":1,"键":2}"#),
        (
            r#"{"x":new ObjectId("a","b","y":2}"#,
            r#"{"x":["a","b"],"y":2}"#,
        ),
        (
            r#"{"x":callback(callback(1,2,"y":3}"#,
            r#"{"x":[1,2],"y":3}"#,
        ),
        (r#"{"x":callback(1,"y":2,"z":3}"#, r#"{"x":1,"y":2,"z":3}"#),
        (r#"{"x":callback(1,"y":}"#, r#"{"x":1,"y":null}"#),
        (
            r#"callback("a:b",{"key":2},[3,4]);"#,
            r#"["a:b",{"key":2},[3,4]]"#,
        ),
        (
            r#"{"x":callback(1,https://example.com}"#,
            r#"{"x":[1,"https://example.com"]}"#,
        ),
        (
            r#"{"x":callback(1,callback(2,"y":3}"#,
            r#"{"x":[1,2],"y":3}"#,
        ),
        (r#"{"x":callback([1],2,"y":3}"#, r#"{"x":[[1],2],"y":3}"#),
        ("[{x:callback(1,y:2}]", r#"[{"x":1,"y":2}]"#),
        (r#"{"x":callback(1,foo)}"#, r#"{"x":[1,"foo"]}"#),
        (
            r#"{"x":callback(1,https://example.com)}"#,
            r#"{"x":[1,"https://example.com"]}"#,
        ),
        (r#"callback(1,foo)"#, r#"[1,"foo"]"#),
        (r#"[callback(1,foo)]"#, r#"[[1,"foo"]]"#),
        (r#"callback(foo,bar)"#, r#"["foo","bar"]"#),
        (r#"callback(foo)"#, r#""foo""#),
        (r#"callback(1,foo);"#, r#"[1,"foo"]"#),
        (r#"callback(1,foo(bar))"#, r#"[1,"foo(bar)"]"#),
        (
            r#"callback(1,https://example.com/path_(a))"#,
            r#"[1,"https://example.com/path_(a)"]"#,
        ),
        (
            r#"callback(1,https://example.com/path_(a(b)))"#,
            r#"[1,"https://example.com/path_(a(b))"]"#,
        ),
        (r#"callback(1,callback(2,foo))"#, r#"[1,[2,"foo"]]"#),
        (r#"callback(1,[foo)])"#, r#"[1,["foo)"]]"#),
        (r#"callback(1,{x:foo)})"#, r#"[1,{"x":"foo)"}]"#),
        (r#"new ObjectId(1,foo)"#, r#"[1,"foo"]"#),
        (r#"NumberLong(1,foo)"#, r#"[1,"foo"]"#),
        (r#"callback(1,值)"#, r#"[1,"值"]"#),
        (r#"callback(1,"foo)")"#, r#"[1,"foo)"]"#),
        (
            r#"callback(1,"https://example.com/a)")"#,
            r#"[1,"https://example.com/a)"]"#,
        ),
        (r#"{"x":callback(1,foo}"#, r#"{"x":[1,"foo"]}"#),
        (r#"[callback(1,foo]"#, r#"[[1,"foo"]]"#),
        (r#"{"x":callback(1,foo,"y":2}"#, r#"{"x":[1,"foo"],"y":2}"#),
        (r#"{x:callback(1,foo,y:2}"#, r#"{"x":[1,"foo"],"y":2}"#),
        (r#"foo)"#, r#""foo)""#),
        (r#"{x:foo)}"#, r#"{"x":"foo)"}"#),
        (r#"[foo)]"#, r#"["foo)"]"#),
        (
            r#"https://example.com/path_(a)"#,
            r#""https://example.com/path_(a)""#,
        ),
        (r#"https://example.com/a)"#, r#""https://example.com/a)""#),
        ("{\"x\":callback(,\"y\":2}", "{\"x\":null,\"y\":2}"),
        ("{x:callback(,y:2}", "{\"x\":null,\"y\":2}"),
        (
            "{\"x\":callback(, /* key */ 'y' /* colon */ :2}",
            "{\"x\":null,  \"y\"  :2}",
        ),
        ("{\"x\":callback(,键:2}", "{\"x\":null,\"键\":2}"),
        ("{\"x\":callback(callback(,\"y\":2}", "{\"x\":null,\"y\":2}"),
        ("{\"x\":new ObjectId(,\"y\":2}", "{\"x\":null,\"y\":2}"),
        ("[{x:callback(,y:2}]", "[{\"x\":null,\"y\":2}]"),
        ("callback(1,2\n{\"a\":3}", "[\n[1,2],\n{\"a\":3}\n]"),
        ("callback(1,2\r\n \t[3,4]", "[\n[1,2],\r\n \t[3,4]\n]"),
        ("callback(1,2\n3", "[\n[1,2],\n3\n]"),
        ("callback(1,2\ntrue", "[\n[1,2],\ntrue\n]"),
        ("callback(1,2\n\"next\"", "[\n[1,2],\n\"next\"\n]"),
        ("callback(1,foo\n{\"a\":3}", "[\n[1,\"foo\"],\n{\"a\":3}\n]"),
        (
            "callback(1,https://example.com/path_(a)\n{\"a\":3}",
            "[\n[1,\"https://example.com/path_(a)\"],\n{\"a\":3}\n]",
        ),
        (
            "callback(1,2 // tail\n{\"a\":3}",
            "[\n[1,2], \n{\"a\":3}\n]",
        ),
        (
            "callback(1,callback(2,3\n{\"a\":4}",
            "[\n[1,[2,3]],\n{\"a\":4}\n]",
        ),
        ("callback(1,2)\n{\"a\":3}", "[\n[1,2],\n{\"a\":3}\n]"),
        ("callback(1\n{\"a\":3}", "[\n1,\n{\"a\":3}\n]"),
        ("callback(1,\n2)", "[1,\n2]"),
        ("callback(1,2\n)", "[1,2\n]"),
        ("callback()", "null"),
    ] {
        for chunk_size in 1..=5 {
            let mut output = Vec::new();
            jsonrepair_reader_to_writer(
                ChunkedReader::new(input.as_bytes(), chunk_size),
                &mut output,
            )
            .unwrap();
            assert_eq!(
                output,
                expected.as_bytes(),
                "input {input:?}, chunk {chunk_size}"
            );
            serde_json::from_slice::<serde_json::Value>(&output).unwrap();
        }
    }
}

#[test]
fn known_wrappers_malformed_arguments_write_no_partial_output() {
    for input in [
        r#"callback(1,foo))"#,
        r#"callback(1,https://example.com))"#,
        r#"callback(1,foo,)"#,
        r#"callback(1,https:/)"#,
        "callback(1,)",
        "callback(1,,2)",
        "callback(1,2 {\"a\":3})",
        "{\"x\":callback(,2}",
        "callback(,\"y\":2)",
        "[callback(,\"y\":2)]",
        "callback(,2)",
        "callback(1,",
        r#"callback(1,"y":2)"#,
        r#"[callback(1,"y":2)]"#,
        r#"[callback({"x":1},"y":2)]"#,
        r#"{"x":callback(1,https:/}"#,
        r#"{"x":callback(1,}"#,
        "[callback(1,]",
        "callback(1,2;)",
        r#"ObjectId("a","\uZZZZ")"#,
    ] {
        let expected = jsonrepair(input).expect_err(input);
        for chunk_size in 1..=5 {
            let mut output = Vec::new();
            let error = jsonrepair_reader_to_writer(
                ChunkedReader::new(input.as_bytes(), chunk_size),
                &mut output,
            )
            .unwrap_err();
            match error {
                JsonRepairStreamError::Repair(error) => assert_eq!(error, expected),
                other => panic!("expected repair error for {input:?}, got {other:?}"),
            }
            assert!(output.is_empty(), "input {input:?}, chunk {chunk_size}");
        }
    }
}

#[test]
fn matches_string_api_for_file_sized_input() {
    let mut lines = Vec::new();
    for index in 0..2048 {
        lines.push(format!("{{id:{index}, name:'item-{index}', ok: True,}}"));
    }
    let input = lines.join("\n");

    let mut reader = ChunkedReader::new(input.as_bytes(), 17);
    let mut output = Vec::new();

    jsonrepair_reader_to_writer(&mut reader, &mut output).unwrap();

    let streamed = String::from_utf8(output).unwrap();
    assert_eq!(streamed, jsonrepair(&input).unwrap());
    serde_json::from_str::<serde_json::Value>(&streamed).unwrap();
}

#[test]
fn chunk_boundary_cases_match_string_api() {
    let cases = [
        r#"{"text":"hello\nworld","quote":"a\"b"}"#,
        "{\n// comment\nname:'Ada', active: True\n}",
        "[.5, 2e, +.5, -Infinity]",
        "{items:[1,2,], name:'Ada',}",
        "{name:'Ada', nested:{items:[1,2,3}",
        "{\"a\":1}\n{\"b\":2}\n[3,4]",
    ];

    for input in cases {
        let expected = jsonrepair(input).unwrap();
        for chunk_size in [1, 2, 3, 5, 8, 13] {
            let mut reader = ChunkedReader::new(input.as_bytes(), chunk_size);
            let mut output = Vec::new();

            jsonrepair_reader_to_writer(&mut reader, &mut output).unwrap();

            assert_eq!(
                String::from_utf8(output).unwrap(),
                expected,
                "input {input:?} with chunk size {chunk_size}",
            );
        }
    }
}

#[test]
fn repairs_truncated_llm_url_across_reader_chunks() {
    let input = r##"{"content":"# Heading\nhttps:/"##;
    let expected = r##"{"content":"# Heading\nhttps:/"}"##;
    for chunk_size in [1, 2, 3, 5] {
        let mut output = Vec::new();
        jsonrepair_reader_to_writer(
            ChunkedReader::new(input.as_bytes(), chunk_size),
            &mut output,
        )
        .unwrap();
        assert_eq!(output, expected.as_bytes(), "chunk size {chunk_size}");
    }
}

#[test]
fn preserves_repair_errors_without_partial_output() {
    for input in [&br#""\u00""#[..], &b"[\x0c"[..], &br#""\udfff""#[..]] {
        let mut output = Vec::new();
        let err = jsonrepair_reader_to_writer(Cursor::new(input), &mut output).unwrap_err();

        assert!(matches!(err, JsonRepairStreamError::Repair(_)));
        assert!(output.is_empty());
    }
}

#[test]
fn reports_read_errors() {
    let mut output = Vec::new();
    let err = jsonrepair_reader_to_writer(FailingReader, &mut output).unwrap_err();

    assert!(matches!(err, JsonRepairStreamError::Read(_)));
    assert!(output.is_empty());
}

#[test]
fn reports_write_errors() {
    let input = Cursor::new("{name: 'Ada'}");
    let mut output = FailingWriter;

    let err = jsonrepair_reader_to_writer(input, &mut output).unwrap_err();

    assert!(matches!(err, JsonRepairStreamError::Write(_)));
}

struct ChunkedReader<'a> {
    input: &'a [u8],
    chunk_size: usize,
    offset: usize,
}

impl<'a> ChunkedReader<'a> {
    fn new(input: &'a [u8], chunk_size: usize) -> Self {
        Self {
            input,
            chunk_size,
            offset: 0,
        }
    }
}

impl Read for ChunkedReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.offset >= self.input.len() {
            return Ok(0);
        }

        let len = self
            .chunk_size
            .min(buf.len())
            .min(self.input.len() - self.offset);
        buf[..len].copy_from_slice(&self.input[self.offset..self.offset + len]);
        self.offset += len;
        Ok(len)
    }
}

struct FailingReader;

impl Read for FailingReader {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::Other, "source closed"))
    }
}

struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::Other, "destination closed"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
