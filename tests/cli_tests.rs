use std::{
    env, fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_jsonrepair")
}

fn temp_path(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    env::temp_dir().join(format!(
        "jsonrepair-rs-{name}-{}-{nanos}",
        std::process::id()
    ))
}

#[test]
fn repairs_stdin_to_stdout() {
    let mut child = Command::new(bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"{name: 'Ada', active: True}")
        .unwrap();

    let output = child.wait_with_output().unwrap();

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        r#"{"name": "Ada", "active": true}"#
    );
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn known_wrappers_multiple_arguments_and_errors() {
    for (input, expected) in [
        (r#"callback({"a":1},2)"#, Some(r#"[{"a":1},2]"#)),
        ("callback(1,2,3)", Some("[1,2,3]")),
        (r#"ObjectId("a","b")"#, Some(r#"["a","b"]"#)),
        ("NumberLong(1,2)", Some("[1,2]")),
        (r#"callback({"a":1},2);"#, Some(r#"[{"a":1},2]"#)),
        ("callback(callback(1,2),3)", Some("[[1,2],3]")),
        (r#"callback(callback({"a":1}),2)"#, Some(r#"[{"a":1},2]"#)),
        (r#"callback({"a":1})"#, Some(r#"{"a":1}"#)),
        (r#"ObjectId("abc")"#, Some(r#""abc""#)),
        (r#"{"x":callback(1,2}"#, Some(r#"{"x":[1,2]}"#)),
        ("[callback(1,2]", Some("[[1,2]]")),
        (
            r#"{"x":callback(callback(1,2),3}"#,
            Some(r#"{"x":[[1,2],3]}"#),
        ),
        (r#"{"x":ObjectId("a","b"}"#, Some(r#"{"x":["a","b"]}"#)),
        (r#"{"x":callback(1}"#, Some(r#"{"x":1}"#)),
        ("[callback(1]", Some("[1]")),
        (r#"{"x":callback(1,"y":2}"#, Some(r#"{"x":1,"y":2}"#)),
        ("{x:callback(1,y:2}", Some(r#"{"x":1,"y":2}"#)),
        (r#"{"x":callback(1,2,"y":3}"#, Some(r#"{"x":[1,2],"y":3}"#)),
        ("{x:callback(1,2,y:3}", Some(r#"{"x":[1,2],"y":3}"#)),
        (
            r#"{"x":callback({"a":1},"y":[2,3]}"#,
            Some(r#"{"x":{"a":1},"y":[2,3]}"#),
        ),
        (
            r#"{"x":callback(1, /* key */ 'y' /* colon */ :2}"#,
            Some(r#"{"x":1,  "y"  :2}"#),
        ),
        (r#"{"x":callback(1,"y\"z":2}"#, Some(r#"{"x":1,"y\"z":2}"#)),
        (r#"{"x":callback(1,键:2}"#, Some(r#"{"x":1,"键":2}"#)),
        (
            r#"{"x":new ObjectId("a","b","y":2}"#,
            Some(r#"{"x":["a","b"],"y":2}"#),
        ),
        (
            r#"{"x":callback(callback(1,2,"y":3}"#,
            Some(r#"{"x":[1,2],"y":3}"#),
        ),
        (
            r#"{"x":callback(1,"y":2,"z":3}"#,
            Some(r#"{"x":1,"y":2,"z":3}"#),
        ),
        (r#"{"x":callback(1,"y":}"#, Some(r#"{"x":1,"y":null}"#)),
        (
            r#"callback("a:b",{"key":2},[3,4]);"#,
            Some(r#"["a:b",{"key":2},[3,4]]"#),
        ),
        (
            r#"{"x":callback(1,https://example.com}"#,
            Some(r#"{"x":[1,"https://example.com"]}"#),
        ),
        (
            r#"{"x":callback(1,callback(2,"y":3}"#,
            Some(r#"{"x":[1,2],"y":3}"#),
        ),
        (
            r#"{"x":callback([1],2,"y":3}"#,
            Some(r#"{"x":[[1],2],"y":3}"#),
        ),
        ("[{x:callback(1,y:2}]", Some(r#"[{"x":1,"y":2}]"#)),
        (r#"{"x":callback(1,foo)}"#, Some(r#"{"x":[1,"foo"]}"#)),
        (
            r#"{"x":callback(1,https://example.com)}"#,
            Some(r#"{"x":[1,"https://example.com"]}"#),
        ),
        (r#"callback(1,foo)"#, Some(r#"[1,"foo"]"#)),
        (r#"[callback(1,foo)]"#, Some(r#"[[1,"foo"]]"#)),
        (r#"callback(foo,bar)"#, Some(r#"["foo","bar"]"#)),
        (r#"callback(foo)"#, Some(r#""foo""#)),
        (r#"callback(1,foo);"#, Some(r#"[1,"foo"]"#)),
        (r#"callback(1,foo(bar))"#, Some(r#"[1,"foo(bar)"]"#)),
        (
            r#"callback(1,https://example.com/path_(a))"#,
            Some(r#"[1,"https://example.com/path_(a)"]"#),
        ),
        (
            r#"callback(1,https://example.com/path_(a(b)))"#,
            Some(r#"[1,"https://example.com/path_(a(b))"]"#),
        ),
        (r#"callback(1,callback(2,foo))"#, Some(r#"[1,[2,"foo"]]"#)),
        (r#"callback(1,[foo)])"#, Some(r#"[1,["foo)"]]"#)),
        (r#"callback(1,{x:foo)})"#, Some(r#"[1,{"x":"foo)"}]"#)),
        (r#"new ObjectId(1,foo)"#, Some(r#"[1,"foo"]"#)),
        (r#"NumberLong(1,foo)"#, Some(r#"[1,"foo"]"#)),
        (r#"callback(1,值)"#, Some(r#"[1,"值"]"#)),
        (r#"callback(1,"foo)")"#, Some(r#"[1,"foo)"]"#)),
        (
            r#"callback(1,"https://example.com/a)")"#,
            Some(r#"[1,"https://example.com/a)"]"#),
        ),
        (r#"{"x":callback(1,foo}"#, Some(r#"{"x":[1,"foo"]}"#)),
        (r#"[callback(1,foo]"#, Some(r#"[[1,"foo"]]"#)),
        (
            r#"{"x":callback(1,foo,"y":2}"#,
            Some(r#"{"x":[1,"foo"],"y":2}"#),
        ),
        (
            r#"{x:callback(1,foo,y:2}"#,
            Some(r#"{"x":[1,"foo"],"y":2}"#),
        ),
        (r#"foo)"#, Some(r#""foo)""#)),
        (r#"{x:foo)}"#, Some(r#"{"x":"foo)"}"#)),
        (r#"[foo)]"#, Some(r#"["foo)"]"#)),
        (
            r#"https://example.com/path_(a)"#,
            Some(r#""https://example.com/path_(a)""#),
        ),
        (
            r#"https://example.com/a)"#,
            Some(r#""https://example.com/a)""#),
        ),
        ("{\"x\":callback(,\"y\":2}", Some("{\"x\":null,\"y\":2}")),
        ("{x:callback(,y:2}", Some("{\"x\":null,\"y\":2}")),
        (
            "{\"x\":callback(, /* key */ 'y' /* colon */ :2}",
            Some("{\"x\":null,  \"y\"  :2}"),
        ),
        ("{\"x\":callback(,键:2}", Some("{\"x\":null,\"键\":2}")),
        (
            "{\"x\":callback(callback(,\"y\":2}",
            Some("{\"x\":null,\"y\":2}"),
        ),
        (
            "{\"x\":new ObjectId(,\"y\":2}",
            Some("{\"x\":null,\"y\":2}"),
        ),
        ("[{x:callback(,y:2}]", Some("[{\"x\":null,\"y\":2}]")),
        ("callback(1,2\n{\"a\":3}", Some("[\n[1,2],\n{\"a\":3}\n]")),
        ("callback(1,2\r\n \t[3,4]", Some("[\n[1,2],\r\n \t[3,4]\n]")),
        ("callback(1,2\n3", Some("[\n[1,2],\n3\n]")),
        ("callback(1,2\ntrue", Some("[\n[1,2],\ntrue\n]")),
        ("callback(1,2\n\"next\"", Some("[\n[1,2],\n\"next\"\n]")),
        (
            "callback(1,foo\n{\"a\":3}",
            Some("[\n[1,\"foo\"],\n{\"a\":3}\n]"),
        ),
        (
            "callback(1,https://example.com/path_(a)\n{\"a\":3}",
            Some("[\n[1,\"https://example.com/path_(a)\"],\n{\"a\":3}\n]"),
        ),
        (
            "callback(1,2 // tail\n{\"a\":3}",
            Some("[\n[1,2], \n{\"a\":3}\n]"),
        ),
        (
            "callback(1,callback(2,3\n{\"a\":4}",
            Some("[\n[1,[2,3]],\n{\"a\":4}\n]"),
        ),
        ("callback(1,2)\n{\"a\":3}", Some("[\n[1,2],\n{\"a\":3}\n]")),
        ("callback(1\n{\"a\":3}", Some("[\n1,\n{\"a\":3}\n]")),
        ("callback(1,\n2)", Some("[1,\n2]")),
        ("callback(1,2\n)", Some("[1,2\n]")),
        ("callback()", Some("null")),
        (r#"callback(1,foo))"#, None),
        (r#"callback(1,https://example.com))"#, None),
        (r#"callback(1,foo,)"#, None),
        (r#"callback(1,https:/)"#, None),
        ("callback(1,)", None),
        ("callback(1,,2)", None),
        ("callback(1,2 {\"a\":3})", None),
        ("{\"x\":callback(,2}", None),
        ("callback(,\"y\":2)", None),
        ("[callback(,\"y\":2)]", None),
        ("callback(,2)", None),
        ("callback(1,", None),
        (r#"callback(1,"y":2)"#, None),
        (r#"[callback(1,"y":2)]"#, None),
        (r#"[callback({"x":1},"y":2)]"#, None),
        (r#"{"x":callback(1,https:/}"#, None),
        (r#"{"x":callback(1,}"#, None),
        ("[callback(1,]", None),
        ("callback(1,2;)", None),
        (r#"ObjectId("a","\uZZZZ")"#, None),
    ] {
        let mut child = Command::new(bin())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();

        if let Some(expected) = expected {
            assert!(output.status.success(), "input {input:?}: {output:?}");
            assert_eq!(output.stdout, expected.as_bytes(), "input: {input:?}");
            serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
            assert!(output.stderr.is_empty(), "{output:?}");
        } else {
            assert_eq!(output.status.code(), Some(1), "input {input:?}: {output:?}");
            assert!(output.stdout.is_empty(), "input {input:?}: {output:?}");
            assert!(String::from_utf8_lossy(&output.stderr).contains("JSON repair error"));
        }
    }
}

#[test]
fn repairs_file_to_output_file() {
    let input_path = temp_path("input");
    let output_path = temp_path("output");
    fs::write(&input_path, "{skills: ['Rust',], ok: False}").unwrap();

    let output = Command::new(bin())
        .arg(&input_path)
        .arg("--output")
        .arg(&output_path)
        .output()
        .unwrap();

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        fs::read_to_string(&output_path).unwrap(),
        r#"{"skills": ["Rust"], "ok": false}"#
    );
    assert!(output.stdout.is_empty(), "{output:?}");

    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn reports_repair_errors() {
    let mut child = Command::new(bin())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(br#""\u00""#)
        .unwrap();

    let output = child.wait_with_output().unwrap();

    assert!(!output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("JSON repair error"));
}

#[test]
fn repair_error_does_not_truncate_existing_output_file() {
    let input_path = temp_path("invalid-input");
    let output_path = temp_path("existing-output");
    fs::write(&input_path, br#""\u00""#).unwrap();
    fs::write(&output_path, "keep me").unwrap();

    let output = Command::new(bin())
        .arg(&input_path)
        .arg("--output")
        .arg(&output_path)
        .output()
        .unwrap();

    assert!(!output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("JSON repair error"));
    assert_eq!(fs::read_to_string(&output_path).unwrap(), "keep me");

    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn help_documents_exit_codes() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(bin()).arg("--help").output()?;

    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Exit Codes:"), "{stdout}");
    assert!(
        stdout.contains("  2                   Command-line usage error."),
        "{stdout}"
    );
    Ok(())
}

#[test]
fn usage_error_exits_2() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(bin()).arg("--unknown").output()?;

    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown option"));
    Ok(())
}

#[test]
fn output_requires_path_exits_2() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(bin()).arg("--output").output()?;

    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("requires a path"));
    Ok(())
}

#[test]
fn dash_reads_stdin_to_output_file() -> Result<(), Box<dyn std::error::Error>> {
    let output_path = temp_path("dash-output");
    let mut child = Command::new(bin())
        .arg("-")
        .arg("--output")
        .arg(&output_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(b"{name: 'Ada', active: True}")?;
    } else {
        return Err(
            std::io::Error::new(std::io::ErrorKind::Other, "stdin pipe was unavailable").into(),
        );
    }

    let output = child.wait_with_output()?;

    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        fs::read_to_string(&output_path)?,
        r#"{"name": "Ada", "active": true}"#
    );

    let _ = fs::remove_file(output_path);
    Ok(())
}

#[test]
fn output_equals_writes_file() -> Result<(), Box<dyn std::error::Error>> {
    let output_path = temp_path("equals-output");
    let output_arg = format!("--output={}", output_path.display());
    let mut child = Command::new(bin())
        .arg(output_arg)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(b"{skills: ['Rust',], ok: False}")?;
    } else {
        return Err(
            std::io::Error::new(std::io::ErrorKind::Other, "stdin pipe was unavailable").into(),
        );
    }

    let output = child.wait_with_output()?;

    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert_eq!(
        fs::read_to_string(&output_path)?,
        r#"{"skills": ["Rust"], "ok": false}"#
    );

    let _ = fs::remove_file(output_path);
    Ok(())
}

#[test]
fn known_wrappers_closed_colon_arguments() {
    for (input, expected) in [
        (
            "{\"x\":callback(1,https://example.com/a//b,a:b)}",
            "{\"x\":[1,\"https://example.com/a//b\",\"a:b\"]}",
        ),
        (
            "{\"x\":callback(1,https://example.com/path_(a)#frag,a:b)}",
            "{\"x\":[1,\"https://example.com/path_(a)#frag\",\"a:b\"]}",
        ),
        ("{\"x\":callback(1,a:b)}", "{\"x\":[1,\"a:b\"]}"),
        ("callback(1,a:b)", "[1,\"a:b\"]"),
        ("[callback(1,a:b)]", "[[1,\"a:b\"]]"),
        ("{\"x\":callback(1,a:b,2)}", "{\"x\":[1,\"a:b\",2]}"),
        (
            "{\"x\":callback(1,a:b(foo),2)}",
            "{\"x\":[1,\"a:b(foo)\",2]}",
        ),
        (
            "{\"x\":callback(1,callback(2,a:b))}",
            "{\"x\":[1,[2,\"a:b\"]]}",
        ),
        (
            "{\"x\":callback(1,a:b),\"y\":2}",
            "{\"x\":[1,\"a:b\"],\"y\":2}",
        ),
        (
            "{\"x\":callback(1,\"y\":\"a)b\"}",
            "{\"x\":1,\"y\":\"a)b\"}",
        ),
        (
            "{\"x\":callback(1,y:foo(bar)}",
            "{\"x\":1,\"y\":\"foo(bar)\"}",
        ),
    ] {
        let mut child = Command::new(bin())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "input {input:?}: {output:?}");
        assert_eq!(output.stdout, expected.as_bytes(), "input {input:?}");
        assert!(output.stderr.is_empty(), "input {input:?}: {output:?}");
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    }
}

#[test]
fn known_wrappers_direct_argument_boundaries() {
    for (input, expected) in [
        (
            "callback(1,\"https://example.com/path_(a))",
            "[1,\"https://example.com/path_(a)\"]",
        ),
        ("callback(1,\"foo(bar))", "[1,\"foo(bar)\"]"),
        ("callback(1,\"foo)", "[1,\"foo\"]"),
        ("callback(1,{\"a\":2)", "[1,{\"a\":2}]"),
        ("callback(1,[2)", "[1,[2]]"),
        ("callback(1,{\"a\":2,)", "[1,{\"a\":2}]"),
        ("callback(1,[2,)", "[1,[2]]"),
        ("{\"x\":callback(1,\"foo)}", "{\"x\":[1,\"foo\"]}"),
        ("[callback(1,{\"a\":2)]", "[[1,{\"a\":2}]]"),
        ("callback(1,\"foo)\")", "[1,\"foo)\"]"),
        ("callback(1,\"foo)bar\")", "[1,\"foo)bar\"]"),
        ("callback(1,\"foo(bar)\")", "[1,\"foo(bar)\"]"),
        (
            "callback(1,\"foo(\\\"bar\\\")\")",
            "[1,\"foo(\\\"bar\\\")\"]",
        ),
        ("callback(1,{\"a\":\"foo)\"})", "[1,{\"a\":\"foo)\"}]"),
        ("callback(1,[\"foo)\"])", "[1,[\"foo)\"]]"),
        ("callback(1,{a:foo)})", "[1,{\"a\":\"foo)\"}]"),
        ("callback(1,[foo)])", "[1,[\"foo)\"]]"),
        ("callback(1,{a:[\"foo)\"]})", "[1,{\"a\":[\"foo)\"]}]"),
        (
            "callback(1,\"https://example.com/path_(a)\")",
            "[1,\"https://example.com/path_(a)\"]",
        ),
    ] {
        let mut child = Command::new(bin())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "input {input:?}: {output:?}");
        assert_eq!(output.stdout, expected.as_bytes(), "input {input:?}");
        assert!(output.stderr.is_empty(), "input {input:?}: {output:?}");
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    }
}

#[test]
fn known_wrappers_equals_properties() {
    for (input, expected) in [
        (r###"{x:callback(1,y=2}"###, r###"{"x":1,"y":2}"###),
        (r###"{"x":callback(1,"y"=2}"###, r###"{"x":1,"y":2}"###),
        (r###"{x:callback(1,'y'=2}"###, r###"{"x":1,"y":2}"###),
        (
            r###"{x:callback(1,y /*key*/ =2}"###,
            r###"{"x":1,"y"  :2}"###,
        ),
        (r###"{x:callback(1,2,y=3}"###, r###"{"x":[1,2],"y":3}"###),
        (r###"{x:callback(,y=2}"###, r###"{"x":null,"y":2}"###),
        (r###"{x:callback(,"y"=2}"###, r###"{"x":null,"y":2}"###),
        (r###"{y=2}"###, r###"{"y":2}"###),
        (r###"{"y"=2}"###, r###"{"y":2}"###),
        (r###"{x:callback(1,y=2)}"###, r###"{"x":[1,"y=2"]}"###),
        (r###"callback(1,y=2)"###, r###"[1,"y=2"]"###),
        (r###"[callback(1,y=2)]"###, r###"[[1,"y=2"]]"###),
        (
            r###"{x:callback(1,https://example.com/?y=2)}"###,
            r###"{"x":[1,"https://example.com/?y=2"]}"###,
        ),
        (r###"{x:callback(1,"y=2")}"###, r###"{"x":[1,"y=2"]}"###),
    ] {
        let mut child = Command::new(bin())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "input {input:?}: {output:?}");
        assert_eq!(output.stdout, expected.as_bytes(), "input {input:?}");
        assert!(output.stderr.is_empty(), "input {input:?}: {output:?}");
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    }
}

#[test]
fn known_wrappers_regex_argument_boundaries() {
    for (input, expected) in [
        (r###"callback(1,/foo)"###, r###"[1,"/foo/"]"###),
        (r###"{x:callback(1,/foo)}"###, r###"{"x":[1,"/foo/"]}"###),
        (r###"[callback(1,/foo)]"###, r###"[[1,"/foo/"]]"###),
        (r###"ObjectId(/foo)"###, r###""/foo/""###),
        (r###"NumberLong(1,/foo)"###, r###"[1,"/foo/"]"###),
        (r###"callback(1,/(foo)/)"###, r###"[1,"/(foo)/"]"###),
        (r###"callback(1,/foo)/)"###, r###"[1,"/foo)/"]"###),
        (r###"callback(1,/foo\))"###, r###"[1,"/foo\\)/"]"###),
        (r###"callback(1,/[(]foo)"###, r###"[1,"/[(]foo/"]"###),
        (r###"callback(1,/[)]foo)"###, r###"[1,"/[)]foo/"]"###),
        (r###"callback(1,/foo(bar))"###, r###"[1,"/foo(bar)/"]"###),
        (r###"callback(1,/[)]/i)"###, r###"[1,"/[)]/i"]"###),
        (r###"callback(1,/foo)bar/i)"###, r###"[1,"/foo)bar/i"]"###),
        (r###"callback(1,/foo\/bar)"###, r###"[1,"/foo\\/bar/"]"###),
        (r###"/foo)"###, r###""/foo)/""###),
        (r###"/foo"###, r###""/foo/""###),
        (r###"/(foo)/"###, r###""/(foo)/""###),
        (r###"/[)]/i"###, r###""/[)]/i""###),
        (r###"{r:/foo)}"###, r###"{"r":"/foo)}/"}"###),
        (r###"callback(1,{r:/foo)/})"###, r###"[1,{"r":"/foo)/"}]"###),
        (r###"callback(1,[/foo)/])"###, r###"[1,["/foo)/"]]"###),
        (
            r###"callback(1,/foo)
{"a":3}"###,
            r###"[
[1,"/foo/"],
{"a":3}
]"###,
        ),
    ] {
        let mut child = Command::new(bin())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "input {input:?}: {output:?}");
        assert_eq!(output.stdout, expected.as_bytes(), "input {input:?}");
        assert!(output.stderr.is_empty(), "input {input:?}: {output:?}");
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    }
}

#[test]
fn known_wrappers_closed_regex_lookahead() {
    for (input, expected) in [
        (
            r###"{x:callback(1,/}/,a:b)}"###,
            r###"{"x":[1,"/}/","a:b"]}"###,
        ),
        (
            r###"{x:callback(1,/]/,a:b)}"###,
            r###"{"x":[1,"/]/","a:b"]}"###,
        ),
        (
            r###"{x:callback(1,/(/,a:b)}"###,
            r###"{"x":[1,"/(/","a:b"]}"###,
        ),
        (
            r###"{x:callback(1,/)/,a:b)}"###,
            r###"{"x":[1,"/)/","a:b"]}"###,
        ),
        (
            r###"{x:callback(1,/[{}()]/i,a:b)}"###,
            r###"{"x":[1,"/[{}()]/i","a:b"]}"###,
        ),
        (
            r###"{x:callback(1,/\}\)\//g,a:b)}"###,
            r###"{"x":[1,"/\\}\\)\\//g","a:b"]}"###,
        ),
        (
            r###"{x:callback(1,/(a)b/,a:b)}"###,
            r###"{"x":[1,"/(a)b/","a:b"]}"###,
        ),
        (r###"callback(1,/}/,a:b)"###, r###"[1,"/}/","a:b"]"###),
        (r###"[callback(1,/}/,a:b)]"###, r###"[[1,"/}/","a:b"]]"###),
        (
            r###"{x:NumberLong(1,/}/,a=b)}"###,
            r###"{"x":[1,"/}/","a=b"]}"###,
        ),
        (
            r###"{x:callback(1,/}/,a:b),y:2}"###,
            r###"{"x":[1,"/}/","a:b"],"y":2}"###,
        ),
        (
            r###"{x:callback(1,/}/,y:2}"###,
            r###"{"x":[1,"/}/"],"y":2}"###,
        ),
        (
            r###"{x:callback(1,/}/,y=2}"###,
            r###"{"x":[1,"/}/"],"y":2}"###,
        ),
    ] {
        let mut child = Command::new(bin())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "input {input:?}: {output:?}");
        assert_eq!(output.stdout, expected.as_bytes(), "input {input:?}");
        assert!(output.stderr.is_empty(), "input {input:?}: {output:?}");
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    }
}

#[test]
fn known_wrappers_quoted_literal_parentheses() {
    for (input, expected) in [
        (r###"callback(1,"foo)bar)"###, r###"[1,"foo)bar"]"###),
        (
            r###"{x:callback(1,"foo)bar)}"###,
            r###"{"x":[1,"foo)bar"]}"###,
        ),
        (r###"[callback(1,"foo)bar)]"###, r###"[[1,"foo)bar"]]"###),
        (r###"ObjectId("foo)bar)"###, r###""foo)bar""###),
        (r###"callback("foo)bar)"###, r###""foo)bar""###),
        (r###"callback(1,"foo)bar);"###, r###"[1,"foo)bar"]"###),
        (
            r###"callback(1,"foo)bar,baz)"###,
            r###"[1,"foo)bar","baz"]"###,
        ),
        (
            r###"callback(1,callback(2,"foo)bar))"###,
            r###"[1,[2,"foo)bar"]]"###,
        ),
        (r###"callback(1,"foo(bar))"###, r###"[1,"foo(bar)"]"###),
        (r###"callback(1,"foo)bar")"###, r###"[1,"foo)bar"]"###),
        (
            r###"callback(1,"foo\"bar)baz)"###,
            r###"[1,"foo\"bar)baz"]"###,
        ),
        (
            r###"callback(1,"https://example.com/foo)bar)"###,
            r###"[1,"https://example.com/foo)bar"]"###,
        ),
        (r###""foo)bar""###, r###""foo)bar""###),
        (r###"{x:"foo)bar}"###, r###"{"x":"foo)bar"}"###),
        (r###"callback(1,"foo)"###, r###"[1,"foo"]"###),
    ] {
        let mut child = Command::new(bin())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "input {input:?}: {output:?}");
        assert_eq!(output.stdout, expected.as_bytes(), "input {input:?}");
        assert!(output.stderr.is_empty(), "input {input:?}: {output:?}");
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    }
}

#[test]
fn known_wrappers_repaired_quote_and_hash_lookahead() {
    for (input, expected) in [
        (
            r###"{x:callback(1,"foo,a:b)}"###,
            r###"{"x":[1,"foo","a:b"]}"###,
        ),
        (r###"callback(1,"foo,a:b)"###, r###"[1,"foo","a:b"]"###),
        (
            r###"{x:callback(1,"foo,a=b)}"###,
            r###"{"x":[1,"foo","a=b"]}"###,
        ),
        (
            r###"{x:callback(1,"foo,a:b),y:2}"###,
            r###"{"x":[1,"foo","a:b"],"y":2}"###,
        ),
        (
            r###"{x:callback(1,"foo,a:b")}"###,
            r###"{"x":[1,"foo,a:b"]}"###,
        ),
        (
            r###"{x:callback(1,"foo\"bar,a:b)}"###,
            r###"{"x":[1,"foo\"bar","a:b"]}"###,
        ),
        (
            r###"{x:callback(1,"foo)bar,a:b)}"###,
            r###"{"x":[1,"foo)bar","a:b"]}"###,
        ),
        (
            r###"{x:callback(1,foo#bar,a:b)}"###,
            r###"{"x":[1,"foo#bar","a:b"]}"###,
        ),
        (
            r###"callback(1,foo#bar,a:b)"###,
            r###"[1,"foo#bar","a:b"]"###,
        ),
        (
            r###"{x:callback(1,foo #bar,a:b)}"###,
            r###"{"x":[1,"foo #bar","a:b"]}"###,
        ),
        (
            r###"{x:callback(1,foo#bar,a=b)}"###,
            r###"{"x":[1,"foo#bar","a=b"]}"###,
        ),
        (
            r###"{x:callback(1,foo#bar,a:b),y:2}"###,
            r###"{"x":[1,"foo#bar","a:b"],"y":2}"###,
        ),
        (
            "{x:callback(1,#comment\nfoo,a:b)}",
            "{\"x\":[1,\n\"foo\",\"a:b\"]}",
        ),
        (
            "{x:callback(1,true #comment\n,a:b)}",
            "{\"x\":[1,true \n,\"a:b\"]}",
        ),
        (
            "{x:callback(1,2 #comment\n,a:b)}",
            "{\"x\":[1,2 \n,\"a:b\"]}",
        ),
        (
            r###"{x:callback(1,foo#bar,y:2}"###,
            r###"{"x":[1,"foo#bar"],"y":2}"###,
        ),
        (
            r###"{x:callback(1,"foo#bar",a:b)}"###,
            r###"{"x":[1,"foo#bar","a:b"]}"###,
        ),
        (
            r###"{x:callback(1,/#}/,a:b)}"###,
            r###"{"x":[1,"/#}/","a:b"]}"###,
        ),
        (
            r###"{x:callback(1,https://example.com/#bar,a:b)}"###,
            r###"{"x":[1,"https://example.com/#bar","a:b"]}"###,
        ),
    ] {
        let mut child = Command::new(bin())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "input {input:?}: {output:?}");
        assert_eq!(output.stdout, expected.as_bytes(), "input {input:?}");
        assert!(output.stderr.is_empty(), "input {input:?}: {output:?}");
    }
}

#[test]
fn known_wrappers_quoted_punctuation_final_close() {
    for (input, expected) in [
        ("callback(\"foo):bar)", "\"foo):bar\""),
        ("callback(1,\"foo):bar)", "[1,\"foo):bar\"]"),
        ("{x:callback(1,\"foo):bar)}", "{\"x\":[1,\"foo):bar\"]}"),
        ("[callback(1,\"foo):bar)]", "[[1,\"foo):bar\"]]"),
        ("ObjectId(\"foo):bar)", "\"foo):bar\""),
        ("callback(1,callback(2,\"foo):bar))", "[1,[2,\"foo):bar\"]]"),
        ("callback(1,\"foo) :bar);", "[1,\"foo) :bar\"]"),
        ("callback(1,\"foo):bar,a:b)", "[1,\"foo):bar\",\"a:b\"]"),
        (
            "{x:callback(1,\"foo):bar,a=b),y:2}",
            "{\"x\":[1,\"foo):bar\",\"a=b\"],\"y\":2}",
        ),
        ("callback(1,\"foo):bar\")", "[1,\"foo):bar\"]"),
        (
            "callback(1,\"foo):bar)\n{\"y\":2}",
            "[\n[1,\"foo):bar\"],\n{\"y\":2}\n]",
        ),
        (
            "{x:callback(1,\"foo):bar),y:\"baz)\"}",
            "{\"x\":[1,\"foo):bar\"],\"y\":\"baz)\"}",
        ),
        ("callback(\"foo) #comment )\n", "\"foo\" \n"),
    ] {
        let mut child = Command::new(bin())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "input {input:?}: {output:?}");
        assert_eq!(output.stdout, expected.as_bytes(), "input {input:?}");
        assert!(output.stderr.is_empty(), "input {input:?}: {output:?}");
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    }
}

#[test]
fn known_wrappers_incomplete_regex_final_close() {
    for (input, expected) in [
        ("callback(/foo)bar)", "\"/foo)bar/\""),
        ("callback(1,/foo)bar)", "[1,\"/foo)bar/\"]"),
        ("{x:callback(1,/foo)bar)}", "{\"x\":[1,\"/foo)bar/\"]}"),
        ("[callback(1,/foo)bar)]", "[[1,\"/foo)bar/\"]]"),
        ("ObjectId(/foo)bar)", "\"/foo)bar/\""),
        ("callback(/foo):bar)", "\"/foo):bar/\""),
        ("callback(/foo)bar)baz)", "\"/foo)bar)baz/\""),
        ("callback(1,callback(2,/foo)bar))", "[1,[2,\"/foo)bar/\"]]"),
        ("callback(callback(/foo),2)", "[\"/foo/\",2]"),
        ("callback(/foo)bar(baz))", "\"/foo)bar(baz)/\""),
        ("callback(/foo)bar[()])", "\"/foo)bar[()]/\""),
        ("callback(/foo)bar\\))", "\"/foo)bar\\\\)/\""),
        ("callback(/foo) #comment )\n", "\"/foo/\" \n"),
        (
            "callback(1,/foo)bar)\n{\"y\":2}",
            "[\n[1,\"/foo)bar/\"],\n{\"y\":2}\n]",
        ),
        ("callback(/foo)bar/i)", "\"/foo)bar/i\""),
        (
            "{x:callback(/foo)bar),y:\"baz)\"}",
            "{\"x\":\"/foo)bar/\",\"y\":\"baz)\"}",
        ),
    ] {
        let mut child = Command::new(bin())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "input {input:?}: {output:?}");
        assert_eq!(output.stdout, expected.as_bytes(), "input {input:?}");
        assert!(output.stderr.is_empty(), "input {input:?}: {output:?}");
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap();
    }
}
