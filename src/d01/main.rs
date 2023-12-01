use itertools::Itertools;
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d01/input")?;
    let numbers = content.lines().map(|line| -> Result<u32, &str> {
        let first_digit = line
            .chars()
            .find(|ch| ch.is_digit(10))
            .ok_or("Couldnt find digit")?
            .to_digit(10)
            .ok_or("Couldnt convert to number")?; // should be impossible
        let last_digit = line
            .chars()
            .rev()
            .find(|ch| ch.is_digit(10))
            .ok_or("Couldnt find digit")?
            .to_digit(10)
            .ok_or("Couldnt convert to number")?; // should be impossible
        Ok(10 * first_digit + last_digit)
    });
    let sum: u32 = numbers.process_results(|iter| iter.sum())?;
    println!("{}", sum);
    Ok(())
}
