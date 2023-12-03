use itertools::iproduct;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;

struct Grid<'a> {
    width: usize,
    height: usize,
    buffer: &'a [char],
}

type Loc = (usize, usize);

impl<'a> Grid<'a> {
    /// given a location (r, c), return all adjacent locations and itself
    fn adjacent_locations(&self, (r, c): Loc) -> Vec<Loc> {
        let (r, c) = (r as isize, c as isize);
        iproduct!([-1, 0, 1], [-1, 0, 1])
            .map(|(dr, dc): (isize, isize)| (r + dr, c + dc))
            .filter(|&(r, c)| {
                0 <= r && r < (self.height as isize) && 0 <= c && c < (self.width as isize)
            })
            .map(|(r, c)| (r as usize, c as usize))
            .collect()
    }
}

/// pointer to the number in the grid
#[derive(Debug)]
struct Number {
    n: usize,
    start_loc: Loc,
    end_loc: Loc,
}
impl Number {
    fn locations(&self) -> impl Iterator<Item = (usize, usize)> {
        let (start_row, start_col) = self.start_loc;
        let (end_row, end_col) = self.end_loc;
        debug_assert!(start_row == end_row);
        (start_col..end_col).map(move |c| (start_row, c))
    }
    fn touches(&self, (r, c): Loc) -> bool {
        let (r, c) = (r as isize, c as isize);

        // take the difference between the given loc an all locations for this number
        let mut differences = self
            .locations()
            .map(|(ri, ci)| ((ri as isize) - r, (ci as isize) - c));

        // return if any of the differences are 1 away
        differences.any(|(dr, dc)| dr.abs() <= 1 && dc.abs() <= 1)
    }
}

impl<'a> Grid<'a> {
    fn get(&self, (r, c): (usize, usize)) -> char {
        debug_assert!(r < self.height);
        debug_assert!(c < self.width);
        // width + 1 to avoid newlines
        self.buffer[(self.width + 1) * r + c]
    }
}

fn get_number_starting_at<'a>(grid: &Grid<'a>, (r, c): (usize, usize)) -> (usize, usize) {
    debug_assert!(grid.get((r, c)).is_digit(10));
    let end_c = (c..=(grid.width))
        .find(|&end_idx| end_idx == grid.width || !grid.get((r, end_idx)).is_digit(10))
        .unwrap();
    let start_idx = r * (grid.width + 1) + c;
    let end_idx = r * (grid.width + 1) + end_c;

    // copies to the heap. Unfortunate
    let n: usize = grid.buffer[start_idx..end_idx]
        .iter()
        .collect::<String>()
        .parse()
        .unwrap();

    (end_c, n)
}

fn main() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d03/input")?;
    let buffer: &[char] = &content.chars().collect::<Vec<char>>();
    let width = buffer
        .iter()
        .enumerate()
        .find_map(|(i, ch)| if ch == &'\n' { Some(i) } else { None })
        .unwrap_or(buffer.len());
    let height = buffer.len() / (width + 1);

    let grid = Grid {
        width,
        height,
        buffer,
    };

    // map from loc -> &Number
    // gears: (loc, number1, number2)
    // for g in gears: calculate ratio and add

    let mut c = 0;
    let mut numbers: Vec<Number> = Vec::new();
    let mut number_map: HashMap<Loc, &Number> = HashMap::new();
    let mut gears: Vec<(Loc, &Number, &Number)> = Vec::new();

    // Popluate numbers
    for r in 0..height {
        loop {
            if grid.get((r, c)).is_digit(10) {
                let (end_c, n) = get_number_starting_at(&grid, (r, c));
                numbers.push(Number {
                    n,
                    start_loc: (r, c),
                    end_loc: (r, end_c),
                });
                c = end_c - 1;
            }
            c += 1;
            debug_assert!(c <= width, "c = {} width = {}", c, width);
            if c >= width {
                break;
            }
        }
        c = 0;
    }

    // Populate gears
    for r in 0..height {
        for c in 0..width {
            if grid.get((r, c)) == '*' {
                let touching_numbers: Vec<&Number> = numbers
                    .iter()
                    .filter(|number| number.touches((r, c)))
                    .collect();
                if touching_numbers.len() == 2 {
                    gears.push(((r, c), touching_numbers[0], touching_numbers[1]));
                }
            }
        }
    }

    // For each gear, multiply its numbers and add them all up
    let n: usize = gears.iter().map(|gear| gear.1.n * gear.2.n).sum();
    println!("{}", n);

    // given a number, look to see if it's touching any symbols
    // let is_touching_symbol = |n: &Number| -> bool {
    //     n.locations()
    //         .flat_map(|(r, c)| adjacent_locations((r as isize, c as isize)))
    //         .any(|loc| symbol_locations.contains(&loc))
    // };

    // let filtered_numbers = numbers.iter().filter(|number| is_touching_symbol(number));

    // let sum: usize = filtered_numbers.map(|number| number.n).sum();
    // println!("{}", sum);

    Ok(())
}
