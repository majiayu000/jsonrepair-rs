#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);

    if let Ok(repaired) = jsonrepair_rs::jsonrepair(&input) {
        serde_json::from_str::<serde_json::Value>(&repaired)
            .expect("successful repairs must be valid JSON");
    }
});
