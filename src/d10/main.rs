use std::error::Error;
use std::fs;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Loc(isize, isize);

impl From<(isize, isize)> for Loc {
    fn from(tuple: (isize, isize)) -> Self {
        Loc(tuple.0, tuple.1)
    }
}

impl From<(usize, usize)> for Loc {
    fn from(tuple: (usize, usize)) -> Self {
        Loc(tuple.0.try_into().unwrap(), tuple.1.try_into().unwrap())
    }
}

impl Loc {
    fn mv(&self, dir: &Direction) -> Self {
        let &Self(r, c) = self;
        match dir {
            Direction::Up => Self(r - 1, c),
            Direction::Down => Self(r + 1, c),
            Direction::Left => Self(r, c - 1),
            Direction::Right => Self(r, c + 1),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Space {
    Start,
    Ground,
    Pipe(Direction, Direction),
}

impl TryFrom<char> for Space {
    type Error = String;

    fn try_from(ch: char) -> Result<Self, Self::Error> {
        use Direction::*;
        use Space::*;
        match ch {
            '|' => Ok(Pipe(Up, Down)),
            '-' => Ok(Pipe(Left, Right)),
            'L' => Ok(Pipe(Up, Right)),
            'J' => Ok(Pipe(Up, Left)),
            '7' => Ok(Pipe(Left, Down)),
            'F' => Ok(Pipe(Down, Right)),
            '.' => Ok(Ground),
            'S' => Ok(Start),
            _ => Err(format!("invalid char {}", ch)),
        }
    }
}
struct Grid {
    width: usize,
    height: usize,
    values: Vec<Space>,
}

impl Grid {
    fn get(&self, loc: Loc) -> Option<Space> {
        let tuple: (Result<usize, _>, Result<usize, _>) = (loc.0.try_into(), loc.1.try_into());
        match tuple {
            (Ok(r), Ok(c)) => self.values.get(self.height * r + c).copied(),
            _ => None,
        }
    }
}

fn parse(content: &str) -> Grid {
    let mut values: Vec<Space> = Vec::new();
    let height = content.lines().count();
    let width = content.lines().nth(0).unwrap().len();
    for line in content.lines() {
        for ch in line.chars() {
            values.push(ch.try_into().unwrap());
        }
    }

    Grid {
        height,
        values,
        width,
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    // parse
    let content = fs::read_to_string("src/d10/input")?;
    let grid = parse(&content);

    // find start_loc
    let (start_idx, _) = grid
        .values
        .iter()
        .enumerate()
        .find(|&(_, space)| space == &Space::Start)
        .unwrap();
    let start_loc: Loc = (start_idx / grid.height, start_idx % grid.height).into();

    // traverse the graph. Fill path: Vec<...> with the path you traveled
    let mut path: Vec<(Loc, usize)> = Vec::new();
    let mut last_loc = start_loc;
    let mut current_loc = last_loc;
    let mut i = 1;
    loop {
        let possible_locations = match grid.get(current_loc) {
            Some(Space::Start) => vec![
                current_loc.mv(&Direction::Up),
                current_loc.mv(&Direction::Down),
                current_loc.mv(&Direction::Left),
                current_loc.mv(&Direction::Right),
            ],
            Some(Space::Pipe(dir1, dir2)) => vec![current_loc.mv(&dir1), current_loc.mv(&dir2)],
            Some(Space::Ground) => panic!("Somehow found yourself on ground"),
            None => panic!("Somehow found yourself off grid"),
        };
        let newloc = *possible_locations
            .iter()
            .find(|&newloc| {
                if *newloc == last_loc {
                    return false;
                }
                match grid.get(*newloc) {
                    None => false,
                    Some(Space::Start) => true,
                    Some(Space::Ground) => false,
                    Some(Space::Pipe(dir1, dir2)) => {
                        newloc.mv(&dir1) == current_loc || newloc.mv(&dir2) == current_loc
                    }
                }
            })
            .unwrap();
        last_loc = current_loc;
        path.push((newloc, i));
        i += 1;
        current_loc = newloc;
        if matches!(grid.get(current_loc), Some(Space::Start)) {
            break;
        }
    }

    // return max(min(i, n - i))
    let m: usize = path
        .iter()
        .map(|&(_, dist)| dist.min(path.len() - dist))
        .max()
        .unwrap();
    println!("max: {}", m);
    Ok(())
}
