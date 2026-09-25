#![no_main]

use libfuzzer_sys::fuzz_target;
use serde::Deserialize;

fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);

    if let Ok(repaired) = jsonrepair_rs::jsonrepair(&input) {
        let mut deserializer = serde_json::Deserializer::from_str(&repaired);
        deserializer.disable_recursion_limit();
        serde_json::Value::deserialize(&mut deserializer)
            .expect("successful repairs must be valid JSON");
        deserializer
            .end()
            .expect("repaired JSON must have no trailing content");
    }
});
