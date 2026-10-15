//! Single-process probe for scripts/memory_report.py; no input generation in RSS.
use jsonrepair_rs::{
    jsonrepair_reader_to_writer_with_options, jsonrepair_to_writer_with_options, RepairOptions,
};
use std::fs::File;
use std::io::{self, BufReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mode = args.next().ok_or("expected string or reader mode")?;
    let path = args.next().ok_or("expected input path")?;
    let options = RepairOptions::new()
        .with_preserve_comment_markers(true)
        .with_decode_unquoted_escapes(true);
    let stdout = io::stdout();
    let mut output = stdout.lock();
    match mode.as_str() {
        "string" => {
            let input = std::fs::read_to_string(path)?;
            jsonrepair_to_writer_with_options(&input, &mut output, options)?;
        }
        "reader" => {
            jsonrepair_reader_to_writer_with_options(
                BufReader::new(File::open(path)?),
                &mut output,
                options,
            )?;
        }
        _ => return Err("expected string or reader mode".into()),
    }
    Ok(())
}
