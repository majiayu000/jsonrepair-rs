use jsonrepair_rs::jsonrepair_parse;

#[derive(Debug, PartialEq, serde::Deserialize)]
struct Profile {
    name: String,
    active: bool,
    skills: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fenced_json = r#"```json
{name: 'Ada', active: True, skills: ['rust', 'json',],}
```"#;

    let profile: Profile = jsonrepair_parse(fenced_json)?;

    assert_eq!(
        profile,
        Profile {
            name: "Ada".to_string(),
            active: true,
            skills: vec!["rust".to_string(), "json".to_string()],
        }
    );

    println!("{profile:?}");
    Ok(())
}
