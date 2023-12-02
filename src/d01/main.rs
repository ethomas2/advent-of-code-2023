use itertools::Itertools;
use std::error::Error;
use std::fs;

fn part1() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d01/input")?;
    let numbers = content.lines().map(|line| -> Result<u32, &str> {
        let first_digit = line
            .chars()
            .find(|ch| ch.is_digit(10))
            .ok_or("Couldnt find digit")?
            .to_digit(10)
            .unwrap();
        let last_digit = line
            .chars()
            .rev()
            .find(|ch| ch.is_digit(10))
            .ok_or("Couldnt find digit")?
            .to_digit(10)
            .unwrap();
        Ok(10 * first_digit + last_digit)
    });
    let sum: u32 = numbers.process_results(|iter| iter.sum())?;
    println!("{}", sum);
    Ok(())
}

const NUMBER_WORDS: [(u32, &str); 10] = [
    (0, "zero"),
    (1, "one"),
    (2, "two"),
    (3, "three"),
    (4, "four"),
    (5, "five"),
    (6, "six"),
    (7, "seven"),
    (8, "eight"),
    (9, "nine"),
];

fn part2() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d01/input")?;
    let line_to_numbers = |line: &str| -> Vec<u32> {
        line.chars()
            .enumerate()
            .flat_map(|(i, ch)| {
                let mut numbers: Vec<u32> = vec![];
                if ch.is_digit(10) {
                    numbers.push(ch.to_digit(10).unwrap());
                } else {
                    for (val, word) in NUMBER_WORDS {
                        if i + word.len() <= line.len() && &line[i..(i + word.len())] == word {
                            numbers.push(val);
                        }
                    }
                }
                numbers
            })
            .collect()
    };
    let parsed_numbers = content.lines().map(|line| {
        let numbers_for_line = line_to_numbers(line);
        10 * numbers_for_line[0] + numbers_for_line.last().unwrap()
    });
    let sum: u32 = parsed_numbers.sum();
    println!("{}", sum);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // part1()
    part2()
}
