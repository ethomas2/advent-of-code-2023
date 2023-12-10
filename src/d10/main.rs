use std::collections::HashSet;
use std::error::Error;
use std::fs;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum PipeType {
    UpDown,
    UpLeft,
    UpRight,
    LeftRight,
    LeftDown,
    RightDown,
}

impl PipeType {
    fn dirs(&self) -> [Direction; 2] {
        match self {
            PipeType::UpDown => [Direction::Up, Direction::Down],
            PipeType::UpLeft => [Direction::Up, Direction::Left],
            PipeType::UpRight => [Direction::Up, Direction::Right],
            PipeType::LeftRight => [Direction::Left, Direction::Right],
            PipeType::LeftDown => [Direction::Left, Direction::Down],
            PipeType::RightDown => [Direction::Right, Direction::Down],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Space {
    Start,
    Ground,
    Pipe(PipeType),
}

impl Space {
    fn to_char(&self) -> char {
        match self {
            Self::Start => 'S',
            Self::Ground => '.',
            Self::Pipe(pt) => pt.to_char(),
        }
    }
}

impl TryFrom<char> for Space {
    type Error = String;

    fn try_from(ch: char) -> Result<Self, Self::Error> {
        use PipeType::*;
        use Space::*;
        match ch {
            '|' => Ok(Pipe(UpDown)),
            '-' => Ok(Pipe(LeftRight)),
            'L' => Ok(Pipe(UpRight)),
            'J' => Ok(Pipe(UpLeft)),
            '7' => Ok(Pipe(LeftDown)),
            'F' => Ok(Pipe(RightDown)),
            '.' => Ok(Ground),
            'S' => Ok(Start),
            _ => Err(format!("invalid char {}", ch)),
        }
    }
}

impl PipeType {
    fn from_dirs((dir1, dir2): (&Direction, &Direction)) -> Self {
        use Direction::*;
        // use PipeType::*;
        match (dir1, dir2) {
            (Up, Down) | (Down, Up) => PipeType::UpDown,
            (Up, Left) | (Left, Up) => PipeType::UpLeft,
            (Up, Right) | (Right, Up) => PipeType::UpRight,
            (Left, Right) | (Right, Left) => PipeType::LeftRight,
            (Left, Down) | (Down, Left) => PipeType::LeftDown,
            (Right, Down) | (Down, Right) => PipeType::RightDown,
            _ => panic!("Unexpected combo"),
        }
    }

    fn to_char(&self) -> char {
        match self {
            Self::UpDown => '|',
            Self::UpLeft => 'J',
            Self::UpRight => 'L',
            Self::LeftRight => '-',
            Self::LeftDown => '7',
            Self::RightDown => 'F',
        }
    }
}

#[derive(Clone)]
struct Grid {
    width: usize,
    height: usize,
    values: Vec<Space>,
}

impl Grid {
    fn get(&self, loc: Loc) -> Option<Space> {
        let tuple: (Result<usize, _>, Result<usize, _>) = (loc.0.try_into(), loc.1.try_into());
        match tuple {
            (Ok(r), Ok(c)) => self.values.get(self.width * r + c).copied(),
            _ => None,
        }
    }

    fn get_mut(&mut self, loc: Loc) -> Option<&mut Space> {
        let tuple: (Result<usize, _>, Result<usize, _>) = (loc.0.try_into(), loc.1.try_into());
        match tuple {
            (Ok(r), Ok(c)) => self.values.get_mut(self.width * r + c),
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

fn get_area(path: &Vec<(Loc, Space)>, grid: &Grid) -> usize {
    use Direction::*;
    use Space::*;

    // mutate the start space in the grid to be a pipe
    let mut grid = grid.clone();
    let (start_idx, &(start_loc, _)) = path
        .iter()
        .enumerate()
        .find(|(_, (_, space))| space == &Start)
        .unwrap();
    let (next_loc, _) = path[(start_idx + 1) % path.len()];
    let (prev_loc, _) = path[(start_idx - 1) % path.len()];
    // TODO: make this a [_; 4]
    let start_dirs: Vec<_> = [Up, Down, Left, Right]
        .iter()
        .filter(|dir| start_loc.mv(dir) == next_loc || start_loc.mv(dir) == prev_loc)
        .collect();
    debug_assert!(start_dirs.len() == 2);
    let start_pipe = PipeType::from_dirs((start_dirs[0], start_dirs[1]));
    *grid.get_mut(start_loc).unwrap() = Pipe(start_pipe);
    let grid = grid;

    let path_locations: HashSet<Loc> = path.iter().map(|&(loc, _)| loc).collect();
    let mut area: usize = 0;
    for r in 0..grid.height {
        let mut inside = false;
        let mut horizontal_entry: Option<Direction> = None;
        for c in 0..grid.width {
            let onpath = path_locations.contains(&(r, c).into());
            match grid.get((r, c).into()).unwrap() {
                Start => panic!("Found Start in mutated path"),
                Ground => {}
                Pipe(pipe_type) if onpath => match pipe_type {
                    PipeType::UpDown => inside = !inside,
                    PipeType::LeftRight => (),
                    // enter a horizontal run
                    PipeType::UpRight => horizontal_entry = Some(Up),
                    PipeType::RightDown => horizontal_entry = Some(Down),
                    // exit a horizontal run
                    PipeType::UpLeft => {
                        if Some(Down) == horizontal_entry {
                            inside = !inside
                        }
                        horizontal_entry = None;
                    }
                    PipeType::LeftDown => {
                        if Some(Up) == horizontal_entry {
                            inside = !inside
                        }
                        horizontal_entry = None;
                    }
                },
                Pipe(_) => {}
            }

            if inside {
                area += 1;
            }
        }
    }
    area
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
    let start_loc: Loc = (start_idx / grid.width, start_idx % grid.width).into();

    // traverse the graph. Fill path: Vec<...> with the path you traveled
    let mut path: Vec<(Loc, Space, usize)> = Vec::new();
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
            Some(Space::Pipe(pt)) => pt.dirs().iter().map(|dir| current_loc.mv(&dir)).collect(),
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
                    Some(Space::Pipe(pt)) => {
                        pt.dirs().iter().any(|dir| newloc.mv(&dir) == current_loc)
                    }
                }
            })
            .unwrap();
        last_loc = current_loc;
        path.push((newloc, grid.get(newloc).unwrap(), i));
        i += 1;
        current_loc = newloc;
        if matches!(grid.get(current_loc), Some(Space::Start)) {
            break;
        }
    }

    // return max(min(i, n - i))
    let m: usize = path
        .iter()
        .map(|&(_, _, dist)| dist.min(path.len() - dist))
        .max()
        .unwrap();
    println!("max: {}", m);
    println!("len {}", path.len());
    // println!("path {:?}", path);

    // part2
    println!(
        "area {}",
        get_area(
            &path.iter().map(|&(loc, space, _)| (loc, space)).collect(),
            &grid
        )
    );
    Ok(())
}
