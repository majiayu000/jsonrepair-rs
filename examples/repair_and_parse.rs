use jsonrepair_rs::jsonrepair;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let broken = r#"{undefined: 1, nested: {ok: True}, regex: /\\z/}"#;
    let repaired = jsonrepair(broken)?;
    let value: serde_json::Value = serde_json::from_str(&repaired)?;

    assert_eq!(value["undefined"], 1);
    assert_eq!(value["nested"]["ok"], true);
    assert_eq!(value["regex"], r"/\\z/");

    println!("repaired: {repaired}");
    println!("parsed keys: {:?}", value.as_object().map(|obj| obj.keys().collect::<Vec<_>>()));
    Ok(())
}
