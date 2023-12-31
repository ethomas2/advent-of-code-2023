use itertools::iproduct;
use std::collections::HashSet;
use std::error::Error;
use std::fs;

struct Grid<'a> {
    width: usize,
    height: usize,
    buffer: &'a [char],
}

/// pointer to the number in the grid
#[derive(Debug)]
struct Number {
    n: usize,
    start_loc: (usize, usize),
    end_loc: (usize, usize),
}
impl Number {
    fn locations(&self) -> impl Iterator<Item = (usize, usize)> {
        let (start_row, start_col) = self.start_loc;
        let (end_row, end_col) = self.end_loc;
        debug_assert!(start_row == end_row);
        (start_col..end_col).map(move |c| (start_row, c))
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
    // 1. Parse the input string into a Grid : a &[char] with a .get(Loc) attribute
    let content = fs::read_to_string("src/d03/input")?;
    let buffer: &[char] = &content.chars().collect::<Vec<char>>();
    let grid: Grid = {
        let width = buffer
            .iter()
            .enumerate()
            .find_map(|(i, ch)| if ch == &'\n' { Some(i) } else { None })
            .unwrap_or(buffer.len());
        let height = buffer.len() / (width + 1);

        Grid {
            width,
            height,
            buffer,
        }
    };

    // 2. Collect all the "Numbers" : a list of structs which are just pointers into the Grid that
    //    has a number
    let numbers: Vec<Number> = {
        let mut c = 0;
        let mut numbers: Vec<Number> = Vec::new();
        for r in 0..grid.height {
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
                debug_assert!(c <= grid.width, "c = {} width = {}", c, grid.width);
                if c >= grid.width {
                    break;
                }
            }
            c = 0;
        }
        numbers
    };

    // 3. Collect locations of all the symbols
    let symbol_locations: HashSet<(usize, usize)> = {
        let mut symbol_locations: HashSet<(usize, usize)> = HashSet::new();
        for r in 0..grid.height {
            for c in 0..grid.width {
                let current_char = grid.get((r, c));
                let is_symbol = !matches!(
                    current_char,
                    '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '.',
                );
                if is_symbol {
                    symbol_locations.insert((r, c));
                }
            }
        }
        symbol_locations
    };

    // given a location (r, c), return all adjacent locations and itself
    let adjacent_locations = |(r, c): (isize, isize)| -> Vec<(usize, usize)> {
        iproduct!([-1, 0, 1], [-1, 0, 1])
            .map(|(dr, dc): (isize, isize)| (r + dr, c + dc))
            .filter(|&(r, c)| {
                0 <= r && r < (grid.height as isize) && 0 <= c && c < (grid.width as isize)
            })
            .map(|(r, c)| (r as usize, c as usize))
            .collect()
    };

    // given a number, look to see if it's touching any symbols
    let is_touching_symbol = |n: &Number| -> bool {
        n.locations()
            .flat_map(|(r, c)| adjacent_locations((r as isize, c as isize)))
            .any(|loc| symbol_locations.contains(&loc))
    };

    // 4. Take the sum of all numbers that touch a symbol
    let filtered_numbers = numbers.iter().filter(|number| is_touching_symbol(number));
    let sum: usize = filtered_numbers.map(|number| number.n).sum();
    println!("{}", sum);

    Ok(())
}
