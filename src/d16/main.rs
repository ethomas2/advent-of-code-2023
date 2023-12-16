#![allow(unused, dead_code)]
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

impl Splitter {
    // TODO: use ArrayVec<2>
    fn split(&self, dir: &Direction) -> Vec<Direction> {
        match (self, dir) {
            (&Splitter::Horizontal, &Direction::Up) => vec![Direction::Left, Direction::Right],
            (&Splitter::Horizontal, &Direction::Down) => vec![Direction::Left, Direction::Right],
            (&Splitter::Horizontal, &Direction::Left) => vec![Direction::Left],
            (&Splitter::Horizontal, &Direction::Right) => vec![Direction::Right],
            (&Splitter::Vertical, &Direction::Up) => vec![Direction::Up],
            (&Splitter::Vertical, &Direction::Down) => vec![Direction::Down],
            (&Splitter::Vertical, &Direction::Left) => vec![Direction::Up, Direction::Down],
            (&Splitter::Vertical, &Direction::Right) => vec![Direction::Up, Direction::Down],
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
                write!(f, "{}", ch);
            }
            writeln!(f);
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
    // TODO: change this to return REsult and get rid of panic! and unwrap()
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
fn main() -> Result<(), Box<dyn Error>> {
    // parse grid
    // grid = Grid<Space>
    // Space = Dot | Reflector { ForwardSlash, BackSlash } | Splitter {Horizontal, Vertical}
    let content = fs::read_to_string("src/d16/input")?;
    let grid = parse(&content);

    // beam_grid = Grid<[Direction; 4]>
    let mut beam_grid: Grid<Vec<Direction>> = Grid {
        width: grid.width,
        height: grid.height,
        items: vec![Vec::new(); grid.items.len()],
    };
    (*beam_grid.get_mut((0, 0).into())).push(Direction::Right);

    // beam_heads = Vec<(Loc, Direction)>
    // TODO: maybe this should be a hashmap. I'm doing contains a lot
    let mut beam_heads: Vec<(Loc, Direction)> = vec![((0, 0).into(), Direction::Right)];
    let mut next_beam_heads = Vec::new();

    // TODO: why does this take so long? profile this
    // loop { for each beam head advance it and mark it in the beam_grid. Break if beam_grid is
    // unchanged }
    'outer: loop {
        // TODO: make arrayvec
        let mut beam_grid_state_changed = false;
        for &(loc, dir) in beam_heads.iter() {
            // get new dirs
            let newdirs = match grid.get(loc) {
                Space::Dot => vec![dir],
                Space::Reflector(r) => vec![r.reflect(&dir)],
                Space::Splitter(s) => s.split(&dir),
            };
            debug_assert!(newdirs.len() >= 1 && newdirs.len() <= 2);

            // advance the new beamheads
            let new_beamheads: Vec<_> = newdirs
                .iter()
                .filter_map(|&newdir| {
                    let newloc = loc.mv(&newdir)?;
                    let _ = grid.contains_loc(newloc).then_some(())?;
                    Some((newloc, newdir))
                })
                .collect();

            // update next next beam heads (for the next iteration)
            next_beam_heads.extend(new_beamheads.iter());

            // update beam_grid
            for (newloc, newdir) in new_beamheads {
                let grid_item = beam_grid.get_mut(newloc);
                if !(*grid_item).contains(&newdir) {
                    (*grid_item).push(newdir);
                    beam_grid_state_changed = true;
                }
            }

            // optional: dedup beam heads
            // TODO: implement
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
    println!("{}", beam_grid.items.iter().filter(|x| x.len() > 0).count());
    Ok(())
}
