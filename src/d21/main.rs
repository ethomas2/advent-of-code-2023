use itertools::Itertools;
use std::collections::HashMap;
use std::error::Error;
use std::fs;

#[derive(Debug, Clone)]
struct Grid<T> {
    items: Vec<T>,
    width: usize,
    height: usize,
}

#[derive(Debug, Clone)]
struct IGrid<T> {
    items: HashMap<ILoc, T>,
    width: usize,  // only used for get_mod
    height: usize, // only used for get_mod
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Space {
    Wall,
    Empty,
    Occupied,
}

type Loc = (usize, usize);

type ILoc = (isize, isize);

impl<T> Grid<T> {
    fn get(&self, (r, c): Loc) -> &T {
        &self.items[self.width * r + c]
    }

    fn get_mut<'a>(&'a mut self, (r, c): Loc) -> &'a mut T {
        &mut self.items[self.width * r + c]
    }
}

impl<T> IGrid<T> {
    fn from_grid<S: Clone>(grid: &Grid<S>) -> IGrid<S> {
        // let mut items = HashMap::with_capacity(grid.items.len());
        let mut items = HashMap::with_capacity(1_000_000);
        for loc in (0..grid.height).cartesian_product(0..grid.width) {
            let (r, c) = loc;
            items.insert(
                (r.try_into().unwrap(), c.try_into().unwrap()),
                grid.get(loc).clone(),
            );
        }
        IGrid {
            items,
            width: grid.width,
            height: grid.height,
        }
    }

    fn get(&self, iloc: ILoc) -> Option<&T> {
        self.items.get(&iloc)
    }

    fn get_mod(&self, (r, c): ILoc) -> &T {
        let (r, c) = (
            modulus(r, self.height.try_into().unwrap()),
            modulus(c, self.width.try_into().unwrap()),
        );
        // let (r, c): (usize, usize) = (r.try_into().unwrap(), c.try_into().unwrap());
        self.items.get(&(r, c)).unwrap()
    }

    fn set(&mut self, iloc: ILoc, val: T) {
        self.items.insert(iloc, val);
    }

    fn iter(&self) -> impl Iterator<Item = (&ILoc, &T)> {
        self.items.iter()
    }

    // fn iter_mut(&mut self) -> impl Iterator<Item = (&ILoc, &mut T)> {
    //     self.items.iter_mut()
    // }
}

fn modulus(x: isize, m: isize) -> isize {
    debug_assert!(m > 0);
    let n = ((x % m) + m) % m;
    debug_assert!(n >= 0);
    n
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

fn expand_no_wrap((r, c): ILoc) -> impl Iterator<Item = ILoc> {
    [(r + 1, c), (r - 1, c), (r, c + 1), (r, c - 1)].into_iter()
}

fn reset(grid: &mut Grid<Space>) {
    for loc in (0..grid.height).cartesian_product(0..grid.width) {
        let space = grid.get_mut(loc);
        if *space == Space::Occupied {
            *space = Space::Empty;
        }
    }
}
fn igrid_reset(igrid: &mut IGrid<Space>) {
    for (_, space) in igrid.items.iter_mut() {
        if space == &Space::Occupied {
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

fn num_occupied_igrid(igrid: &IGrid<Space>) -> usize {
    igrid
        .iter()
        .filter(|(_, space)| *space == &Space::Occupied)
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
    let grid = parse(&input)?;

    // part 1
    {
        let times = 64;
        let mut current_grid = grid.clone();
        let mut next_grid = grid.clone();

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
                        // TODO: use entry
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
    }

    //part 2
    {
        // TODO: don't want to have to do ::<Space>
        let igrid = IGrid::<Space>::from_grid(&grid);

        let mut current_grid = igrid.clone();
        let mut next_grid = igrid.clone();

        // Refer to next_grid and current_grid only by pointers so you can swap without copying
        let mut current_grid = &mut current_grid;
        let mut next_grid = &mut next_grid;
        igrid_reset(&mut next_grid);

        let sequence = (1..10000).map(|i| {
            for (&iloc, _) in current_grid.iter() {
                if current_grid.get(iloc) == Some(&Space::Occupied) {
                    for (rr, cc) in expand_no_wrap(iloc) {
                        if next_grid.get_mod((rr, cc)) != &Space::Wall {
                            next_grid.set((rr, cc), Space::Occupied);
                        }
                    }
                }
            }
            let n = num_occupied_igrid(&next_grid);

            std::mem::swap(&mut current_grid, &mut next_grid);
            igrid_reset(next_grid);
            (i, n)
        });

        // let sequence: Vec<_> = sequence.take(500).collect();

        let sequence: Vec<_> = sequence.take(2000).collect();

        // let first_order_diff: Vec<_> = sequence
        //     .iter()
        //     .tuple_windows()
        //     .map(|(&x1, &x2)| {
        //         let (_, n1) = x1;
        //         let (i, n2) = x2;
        //         let (n1, n2): (isize, isize) = (n1.try_into().unwrap(), n2.try_into().unwrap());
        //         (i, n2 - n1)
        //     })
        //     .collect();

        // let second_order_diff: Vec<_> = first_order_diff
        //     .iter()
        //     .tuple_windows()
        //     .map(|(&x1, &x2)| {
        //         let (_, n1) = x1;
        //         let (i, n2) = x2;
        //         (i, n2 - n1)
        //     })
        //     .collect();

        println!("{:?}", sequence.iter().map(|(_, n)| n).collect::<Vec<_>>());

        // println!("========================================================================");
        // println!(
        //     "1st order {:?}",
        //     first_order_diff.iter().map(|(_, n)| n).collect::<Vec<_>>()
        // );

        // println!("========================================================================");
        // println!(
        //     "2nd order {:?}",
        //     second_order_diff.iter().map(|(_, n)| n).collect::<Vec<_>>()
        // );

        // println!("========================================================================");
        // for (i, n) in sequence {
        //     if [6, 10, 50, 100, 500].contains(&i) {
        //         println!("i::{}  n::{}", i, n);
        //     }
        // }
    }
    Ok(())
}
