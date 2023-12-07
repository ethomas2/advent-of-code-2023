use std::error::Error;
use std::fs;

fn perfect_sqrt(n: u64) -> (f64, bool) {
    let sqrt = (n as f64).sqrt();
    let sqrt_as_int = sqrt.floor() as u64;
    if sqrt_as_int * sqrt_as_int == n {
        return (sqrt, true);
    }
    return (sqrt, false);
}

fn main() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d06/input")?;
    let lines: Vec<&str> = content.lines().collect();

    let times = &lines[0]
        .split_whitespace()
        .skip(1)
        .map(|x| x.parse::<u64>())
        .collect::<Result<Vec<u64>, _>>()?;

    let distances = &lines[1]
        .split_whitespace()
        .skip(1)
        .map(|x| x.parse::<u64>())
        .collect::<Result<Vec<u64>, _>>()?;

    // uncomment for part 2
    // let times = [41968894u64];
    // let distances = [214178911271055u64];

    let product: u64 = times
        .iter()
        .zip(distances.iter())
        .map(|(&T, &D)| {
            let discriminant = dbg!(T * T - 4 * D);
            let (sqrt_discriminant, is_perfect_sqrt) = dbg!(perfect_sqrt(discriminant));
            let mut x1: f64 = ((T as f64) - sqrt_discriminant) / 2f64;
            let mut x2: f64 = ((T as f64) + sqrt_discriminant) / 2f64;
            if is_perfect_sqrt && (T - (sqrt_discriminant as u64)) % 2 == 0 {
                x1 += 1f64;
                x2 -= 1f64;
            }
            let min = u64::min(T, x1.ceil() as u64);
            let max = u64::min(T, x2.floor() as u64);

            let nways = max - min + 1;
            nways
        })
        .product();
    println!("product {}", product);
    Ok(())
}
