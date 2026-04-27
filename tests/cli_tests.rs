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
