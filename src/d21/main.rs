use itertools::Itertools;
use std::error::Error;
use std::fs;

#[derive(Debug, Clone)]
struct Grid<T> {
    items: Vec<T>,
    width: usize,
    height: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Space {
    Wall,
    Empty,
    Occupied,
}

type Loc = (usize, usize);

impl<T> Grid<T> {
    fn get(&self, (r, c): Loc) -> &T {
        &self.items[self.width * r + c]
    }

    fn get_mut<'a>(&'a mut self, (r, c): Loc) -> &'a mut T {
        &mut self.items[self.width * r + c]
    }
}

fn parse(input: &str) -> Result<Grid<Space>, String> {
    let items: Vec<_> = input
        .lines()
        .flat_map(|line| {
            line.chars().map(|ch| match ch {
                '.' => Space::Empty,
                'S' => Space::Occupied,
                '#' => Space::Wall,
                _ => panic!("unknown char {}", ch), // TODO: don't panic, return Err
            })
        })
        .collect();
    let width = input.lines().nth(0).unwrap().len(); // TODO: don't unwrap
    let height = input.lines().count();
    let grid = Grid {
        items,
        width,
        height,
    };

    Ok(grid)
}

fn expand((r, c): Loc, width: usize, height: usize) -> impl Iterator<Item = Loc> {
    let (r, c): (isize, isize) = (r.try_into().unwrap(), c.try_into().unwrap());
    [(r + 1, c), (r - 1, c), (r, c + 1), (r, c - 1)]
        .into_iter()
        .filter(move |&(r, c)| 0 <= r && 0 <= c && r < height as isize && c < width as isize)
        .map(|(r, c)| (r.try_into().unwrap(), c.try_into().unwrap()))
}

fn reset(grid: &mut Grid<Space>) {
    for loc in (0..grid.height).cartesian_product(0..grid.width) {
        let space = grid.get_mut(loc);
        if *space == Space::Occupied {
            *space = Space::Empty;
        }
    }
}

fn occupied_locs(grid: &Grid<Space>) -> impl Iterator<Item = Loc> + '_ {
    (0..grid.height)
        .cartesian_product(0..grid.width)
        .filter(|loc| grid.get(*loc) == &Space::Occupied)
}

fn num_occupied(grid: &Grid<Space>) -> usize {
    (0..grid.height)
        .cartesian_product(0..grid.width)
        .filter(|loc| grid.get(*loc) == &Space::Occupied)
        .count()
}

fn main() -> Result<(), Box<dyn Error>> {
    // parse to grid
    // current grid, next grid
    // for _ in 16
    //      for loc in grid
    //           EXPAND(grid) -> nextgrid
    //      current_grid = next_grid
    // count number of Os in grid

    let input = fs::read_to_string("src/d21/input")?;
    let times = 64;
    let mut current_grid = parse(&input)?;
    let mut next_grid = current_grid.clone();

    // Refer to next_grid and current_grid only by pointers so you can swap without copying
    let mut current_grid = &mut current_grid;
    let mut next_grid = &mut next_grid;
    reset(next_grid);

    let Grid { width, height, .. } = current_grid;
    let (width, height) = (*width, *height);

    for _ in 0..times {
        for loc in (0..height).cartesian_product(0..width) {
            if *current_grid.get(loc) == Space::Occupied {
                for (rr, cc) in expand(loc, width, height) {
                    if next_grid.get((rr, cc)) != &Space::Wall {
                        *next_grid.get_mut((rr, cc)) = Space::Occupied;
                    }
                }
            }
        }
        std::mem::swap(&mut current_grid, &mut next_grid);
        reset(next_grid);
    }
    println!("p1 {}", num_occupied(current_grid));
    Ok(())
}
