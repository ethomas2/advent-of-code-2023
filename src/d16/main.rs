use arrayvec::ArrayVec;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs;
use std::mem;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Loc(usize, usize);
impl Loc {
    fn mv(&self, dir: &Direction) -> Option<Loc> {
        let Loc(r, c) = *self;
        match dir {
            Direction::Up => r.checked_sub(1).map(|rnew| Loc(rnew, c)),
            Direction::Down => Some(Loc(r + 1, c)),
            Direction::Left => c.checked_sub(1).map(|cnew| Loc(r, cnew)),
            Direction::Right => Some(Loc(r, c + 1)),
        }
    }
}

impl From<(usize, usize)> for Loc {
    fn from((r, c): (usize, usize)) -> Self {
        Loc(r, c)
    }
}

enum Reflector {
    ForwardSlash,
    BackSlash,
}

impl Reflector {
    fn reflect(&self, dir: &Direction) -> Direction {
        match (self, dir) {
            (&Reflector::ForwardSlash, &Direction::Up) => Direction::Right,
            (&Reflector::ForwardSlash, &Direction::Down) => Direction::Left,
            (&Reflector::ForwardSlash, &Direction::Left) => Direction::Down,
            (&Reflector::ForwardSlash, &Direction::Right) => Direction::Up,
            (&Reflector::BackSlash, &Direction::Up) => Direction::Left,
            (&Reflector::BackSlash, &Direction::Down) => Direction::Right,
            (&Reflector::BackSlash, &Direction::Left) => Direction::Up,
            (&Reflector::BackSlash, &Direction::Right) => Direction::Down,
        }
    }
}

enum Splitter {
    Horizontal,
    Vertical,
}

macro_rules! avec {
    ($($dir:expr),*) => {{
        let mut vec = ArrayVec::<Direction, 2>::new();
        $( vec.push($dir); )*
        vec
    }};
}

impl Splitter {
    fn split(&self, dir: &Direction) -> ArrayVec<Direction, 2> {
        match (self, dir) {
            (Splitter::Horizontal, Direction::Up | Direction::Down) => {
                avec!(Direction::Left, Direction::Right)
            }
            (Splitter::Horizontal, Direction::Left) => {
                avec!(Direction::Left)
            }
            (Splitter::Horizontal, Direction::Right) => {
                avec!(Direction::Right)
            }
            (Splitter::Vertical, Direction::Up) => {
                avec!(Direction::Up)
            }
            (Splitter::Vertical, Direction::Down) => {
                avec!(Direction::Down)
            }
            (Splitter::Vertical, Direction::Left | Direction::Right) => {
                avec!(Direction::Up, Direction::Down)
            }
        }
    }
}

enum Space {
    Dot,
    Reflector(Reflector),
    Splitter(Splitter),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Grid<T> {
    width: usize,
    height: usize,
    items: Vec<T>,
}

impl<T> Display for Grid<Vec<T>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for r in 0..self.height {
            for c in 0..self.width {
                let x = self.get((r, c).into());
                let ch = if x.len() == 0 { '.' } else { '#' };
                write!(f, "{}", ch)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl<T> Grid<T> {
    fn get(&self, Loc(r, c): Loc) -> &T {
        &self.items[r * self.width + c]
    }

    fn get_mut(&mut self, Loc(r, c): Loc) -> &mut T {
        &mut self.items[r * self.width + c]
    }
    fn contains_loc(&self, Loc(r, c): Loc) -> bool {
        let &Grid { width, height, .. } = self;
        0 <= r && r < height && 0 <= c && c < width
    }
}

fn parse(content: &str) -> Grid<Space> {
    // TODO: change this to return Result and get rid of panic! and unwrap()
    let items = Vec::from_iter(
        content
            .lines()
            .flat_map(|line| line.chars())
            .map(|ch| match ch {
                '.' => Space::Dot,
                '|' => Space::Splitter(Splitter::Vertical),
                '-' => Space::Splitter(Splitter::Horizontal),
                '/' => Space::Reflector(Reflector::ForwardSlash),
                '\\' => Space::Reflector(Reflector::BackSlash),
                _ => panic!("unexpected character \"{}\"", ch),
            }),
    );
    let width = content.lines().nth(0).unwrap().len();
    let height = content.lines().count();
    Grid {
        width,
        height,
        items,
    }
}

fn part1(grid: &Grid<Space>, startloc: Loc, startdir: Direction) -> usize {
    // beam_grid = Grid<[Direction; 4]>
    let mut beam_grid: Grid<ArrayVec<Direction, 4>> = Grid {
        width: grid.width,
        height: grid.height,
        items: vec![ArrayVec::new(); grid.items.len()],
    };
    (*beam_grid.get_mut(startloc)).push(Direction::Right);

    // beam_heads = Vec<(Loc, Direction)>
    let mut beam_heads: Vec<(Loc, Direction)> = vec![(startloc, startdir)];
    let mut next_beam_heads = Vec::new();

    'outer: loop {
        let mut beam_grid_state_changed = false;
        for &(loc, dir) in beam_heads.iter() {
            // get new dirs
            let newdirs = match grid.get(loc) {
                Space::Dot => avec![dir],
                Space::Reflector(r) => avec![r.reflect(&dir)],
                Space::Splitter(s) => s.split(&dir),
            };
            debug_assert!(newdirs.len() >= 1 && newdirs.len() <= 2);

            // advance the new beamheads
            let new_beamheads: ArrayVec<(Loc, Direction), 2> = {
                let mut new_beamheads: ArrayVec<(Loc, Direction), 2> = ArrayVec::new();
                newdirs
                    .iter()
                    .filter_map(|&newdir| {
                        let newloc = loc.mv(&newdir)?;
                        let _ = grid.contains_loc(newloc).then_some(())?;
                        Some((newloc, newdir))
                    })
                    .for_each(|x| new_beamheads.push(x));
                new_beamheads
            };

            // update next next beam heads (for the next iteration)
            next_beam_heads.extend(new_beamheads.iter());

            // update beam_grid
            for &(newloc, newdir) in new_beamheads.iter() {
                let grid_item = beam_grid.get_mut(newloc);
                if !(*grid_item).contains(&newdir) {
                    (*grid_item).push(newdir);
                    beam_grid_state_changed = true;
                }
            }
        }

        // break if beam grid unchanged
        if !beam_grid_state_changed {
            break 'outer;
        }
        mem::swap(&mut beam_heads, &mut next_beam_heads);
        next_beam_heads.truncate(0);
        if !beam_grid_state_changed {
            break;
        }
    }
    beam_grid.items.iter().filter(|x| x.len() > 0).count()
}

fn part2(grid: &Grid<Space>) -> usize {
    let mut answer = 0;
    for c in 0..grid.width {
        // top down
        answer = answer.max(part1(grid, (0, c).into(), Direction::Down));

        // bottom up
        answer = answer.max(part1(grid, (grid.height - 1, c).into(), Direction::Up));

        if c % 20 == 0 && c > 0 {
            println!("Done with columns {}", c);
        }
    }

    for r in 0..grid.height {
        // left to right
        answer = answer.max(part1(grid, (r, 0).into(), Direction::Right));

        // right to left
        answer = answer.max(part1(grid, (r, grid.width - 1).into(), Direction::Left));

        if r % 20 == 0 && r > 0 {
            println!("Done with rows {}", r);
        }
    }
    answer
}

fn main() -> Result<(), Box<dyn Error>> {
    // parse grid
    // grid = Grid<Space>
    // Space = Dot | Reflector { ForwardSlash, BackSlash } | Splitter {Horizontal, Vertical}
    let content = fs::read_to_string("src/d16/input")?;
    let grid = parse(&content);
    let part1_answer = part1(&grid, (0, 0).into(), Direction::Right);
    println!("part1 :: {}", part1_answer);

    let part2_answer = part2(&grid);
    println!("part2 :: {}", part2_answer);

    Ok(())
}
