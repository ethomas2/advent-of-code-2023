use std::error::Error;
use std::fs;

fn predict_next(sequence: &Vec<i64>) -> i64 {
    let differences: Vec<i64> = sequence.windows(2).map(|w| w[1] - w[0]).collect();

    if differences.iter().all(|&d| d == 0) {
        return *sequence.last().unwrap();
    }
    let next_diff = predict_next(&differences);
    return sequence.last().unwrap() + next_diff;
}

fn main() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d09/input")?;
    let mut sequences: Vec<Vec<i64>> = content
        .lines()
        .map(|line| -> Result<Vec<i64>, std::num::ParseIntError> {
            let sequence: Vec<i64> = line
                .split(' ')
                .map(|item| {
                    let x = item.parse::<i64>();
                    x
                })
                .collect::<Result<Vec<i64>, _>>()?;
            Ok(sequence)
        })
        .collect::<Result<Vec<Vec<i64>>, _>>()?;

    let s: i64 = sequences
        .iter()
        .map(|sequence| predict_next(&sequence))
        .sum();

    println!("part 1 {}", s);

    // part 2
    sequences.iter_mut().for_each(|sequence| sequence.reverse());

    let s2: i64 = sequences
        .iter()
        .map(|sequence| predict_next(&sequence))
        .sum();

    println!("part 2 {}", s2);
    Ok(())
}
