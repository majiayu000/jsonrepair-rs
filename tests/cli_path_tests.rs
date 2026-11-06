#![cfg(target_os = "linux")]

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

// Linux filesystems permit these raw byte names; macOS/APFS rejects them with EILSEQ.
#[test]
fn output_equals_preserves_non_utf8_paths_from_stdin_and_files() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    let directory = temp_path("non-utf8-output");
    fs::create_dir(&directory).unwrap();
    let input_path = directory.join("input.json");
    fs::write(&input_path, b"{name: 'Ada'}").unwrap();

    for from_stdin in [true, false] {
        for suffix in [vec![0xFF], vec![0xFE], "\u{FFFD}".as_bytes().to_vec()] {
            let mut name = if from_stdin {
                b"stdin-".to_vec()
            } else {
                b"file-".to_vec()
            };
            name.extend_from_slice(&suffix);
            name.extend_from_slice(b"=result.json");
            let path = directory.join(OsString::from_vec(name));
            let mut argument = OsString::from("--output=");
            argument.push(&path);
            let mut command = Command::new(bin());
            command.arg(argument);
            if !from_stdin {
                command.arg(&input_path);
            }
            let mut child = command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            if from_stdin {
                child
                    .stdin
                    .take()
                    .unwrap()
                    .write_all(b"{name: 'Ada'}")
                    .unwrap();
            }
            let output = child.wait_with_output().unwrap();
            assert!(output.status.success(), "{output:?}");
            assert!(output.stdout.is_empty(), "{output:?}");
            assert!(output.stderr.is_empty(), "{output:?}");
            assert_eq!(fs::read(&path).unwrap(), br#"{"name": "Ada"}"#);
        }
    }
    assert_eq!(fs::read_dir(&directory).unwrap().count(), 7);
    fs::remove_dir_all(directory).unwrap();
}
