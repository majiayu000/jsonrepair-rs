use jsonrepair_rs::jsonrepair;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let llm_output = r#"The model returned this object:
{name: 'Ada', active: True, skills: ['rust', 'json',],}
Use it for the profile card."#;

    let repaired = jsonrepair(llm_output)?;
    let value: serde_json::Value = serde_json::from_str(&repaired)?;

    let profile = value
        .as_array()
        .and_then(|items| items.iter().find(|item| item.get("name").is_some()))
        .expect("repaired LLM output should contain the profile object");

    assert_eq!(profile["name"], "Ada");
    assert_eq!(profile["active"], true);
    assert_eq!(profile["skills"][0], "rust");

    println!("{repaired}");
    Ok(())
}
