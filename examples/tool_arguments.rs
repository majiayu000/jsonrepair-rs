//! Run with: cargo run --example tool_arguments
//! Synthetic completed payload shaped like OpenBitFun's string-repair call.
use jsonrepair_rs::{
    jsonrepair_reader_to_writer_with_options, jsonrepair_with_options, RepairOptions,
};
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let completed_arguments = r##"{"content": # Heading\n// code\n/* example */"}"##;
    let options = RepairOptions::new()
        .with_preserve_comment_markers(true)
        .with_decode_unquoted_escapes(true)
        .with_max_input_bytes(1024 * 1024);
    let candidate = jsonrepair_with_options(completed_arguments, options)?;
    let arguments: serde_json::Value = serde_json::from_str(&candidate)?;
    assert_eq!(arguments["content"], "# Heading\n// code\n/* example */");
    // The owning tool must also validate its schema and important content.
    // This IO helper buffers full input/output and waits for EOF; it does not
    // incrementally decode a live tool-argument stream.
    let mut output = Vec::new();
    jsonrepair_reader_to_writer_with_options(
        Cursor::new(completed_arguments),
        &mut output,
        options,
    )?;
    assert_eq!(output, candidate.as_bytes());
    Ok(())
}
