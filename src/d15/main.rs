use std::error::Error;
use std::fs;
use std::str::FromStr;

fn hash(input: &str) -> u8 {
    let mut value: u8 = 0;
    for ch in input.chars() {
        value = 17u8.wrapping_mul(value.wrapping_add(ch as u8));
    }
    value
}

enum Instruction<'a> {
    Set { label: &'a str, val: u8 },
    Del(&'a str),
}

fn main() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d15/input")?;
    let inputs = content.trim().split(",");
    let s1: usize = inputs.map(|x| hash(x) as usize).sum();
    println!("s1 {}", s1);
    Ok(())
}
