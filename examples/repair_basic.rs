use jsonrepair_rs::jsonrepair;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let broken = r#"{name: 'Alice', active: True, skills: ['Rust',],}"#;
    let repaired = jsonrepair(broken)?;

    println!("input: {broken}");
    println!("repaired: {repaired}");
    Ok(())
}
