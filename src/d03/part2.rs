use std::error::Error;
use std::fs;

struct Grid<'a> {
    width: usize,
    height: usize,
    buffer: &'a [char],
}

type Loc = (usize, usize);

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
        let r: isize = r.try_into().unwrap();
        let c: isize = c.try_into().unwrap();

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

    // 3. Collect a list of all the "Gears" : any '*' in the grid that touches exactly 2 numbers
    // Populate gears
    let gears: Vec<(Loc, &Number, &Number)> = {
        let mut gears = Vec::new();
        for r in 0..grid.height {
            for c in 0..grid.width {
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
        gears
    };

    // 4. For each gear, calculate the product of its two adjacent numbers and sum it all up
    let n: usize = gears.iter().map(|gear| gear.1.n * gear.2.n).sum();
    println!("{}", n);

    Ok(())
}
