use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d11/input")?;
    let height = content.lines().count();
    let width = content.lines().nth(0).unwrap().len();
    let mut v = Vec::with_capacity(width * height);
    let mut empty_rows = Vec::with_capacity(height);
    let mut empty_cols = Vec::with_capacity(width);
    let mut locations = Vec::new();
    content.lines().enumerate().for_each(|(r, line)| {
        line.chars().enumerate().for_each(|(c, ch)| {
            v.push(ch);
            if ch == '#' {
                locations.push((r, c))
            }
        })
    });
    for r in 0..height {
        if (0..width).all(|c| v[r * width + c] == '.') {
            empty_rows.push(r);
        }
    }
    for c in 0..width {
        if (0..height).all(|r| v[r * width + c] == '.') {
            empty_cols.push(c);
        }
    }

    let mut s: usize = 0;
    for i in 0..locations.len() {
        for j in (i + 1)..locations.len() {
            let (r1, c1) = locations[i];
            let (r2, c2) = locations[j];
            let manhattan_distance = r1.abs_diff(r2) + c1.abs_diff(c2);
            let extra_distance = empty_rows
                .iter()
                .filter(|&&r| r1.min(r2) < r && r < r1.max(r2))
                .count()
                + empty_cols
                    .iter()
                    .filter(|&&c| c1.min(c2) < c && c < c1.max(c2))
                    .count();
            s += manhattan_distance + (1000000 - 1) * extra_distance;
        }
    }

    println!("s {}", s);

    Ok(())
}
