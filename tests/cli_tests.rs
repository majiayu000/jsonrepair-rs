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
        ("callback()", Some("null")),
        ("callback(1,)", None),
        ("callback(1,,2)", None),
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
