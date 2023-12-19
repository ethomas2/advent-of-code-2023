use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::hash::Hash;

struct Grid {
    locations: HashSet<Loc>,
    minr: isize,
    maxr: isize,
    minc: isize,
    maxc: isize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RawSpace {
    Dot,
    Hash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Corner {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Flat {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Space {
    Dot,
    Corner(Corner),
    Flat(Flat),
}

impl Grid {
    fn parse(content: &str) -> Result<Self, Box<dyn Error>> {
        let instructions = parse(&content)?;
        let locations = {
            let mut locations: HashSet<Loc> = HashSet::new();
            let mut start: Loc = Loc(0, 0);
            locations.insert(start);
            instructions.iter().for_each(|instruction| {
                let Instruction { direction, num } = *instruction;
                for _ in 0..num {
                    start = start.mv(direction);
                    locations.insert(start);
                }
            });
            locations
        };

        let (minr, maxr, minc, maxc) = {
            let (mut minr, mut maxr, mut minc, mut maxc) = (0, 0, 0, 0);
            for &Loc(r, c) in locations.iter() {
                minr = minr.min(r);
                maxr = maxr.max(r);
                minc = minc.min(c);
                maxc = maxc.max(c);
            }
            (minr, maxr, minc, maxc)
        };

        Ok(Grid {
            locations,
            minr,
            maxr,
            minc,
            maxc,
        })
    }

    fn get_raw(&self, loc: Loc) -> RawSpace {
        match &self.locations.contains(&loc) {
            true => RawSpace::Hash,
            false => RawSpace::Dot,
        }
    }

    fn get(&self, loc: Loc) -> Space {
        match self.get_raw(loc) {
            RawSpace::Dot => Space::Dot,
            RawSpace::Hash => {
                let (up, down, left, right) = (
                    self.get_raw(loc.mv(Direction::Up)),
                    self.get_raw(loc.mv(Direction::Down)),
                    self.get_raw(loc.mv(Direction::Left)),
                    self.get_raw(loc.mv(Direction::Right)),
                );
                if up == RawSpace::Hash && down == RawSpace::Hash {
                    Space::Flat(Flat::Vertical)
                } else if right == RawSpace::Hash && left == RawSpace::Hash {
                    Space::Flat(Flat::Horizontal)
                } else if up == RawSpace::Hash && left == RawSpace::Hash {
                    Space::Corner(Corner::TopLeft)
                } else if up == RawSpace::Hash && right == RawSpace::Hash {
                    Space::Corner(Corner::TopRight)
                } else if down == RawSpace::Hash && left == RawSpace::Hash {
                    Space::Corner(Corner::BottomLeft)
                } else if down == RawSpace::Hash && right == RawSpace::Hash {
                    Space::Corner(Corner::BottomRight)
                } else {
                    panic!("oh no");
                }
            }
        }
    }

    fn get_area(&self) -> usize {
        // sweep from left -> right
        //
        // if vert wall: flip
        // if corner:
        //     - set state last corner
        // if horizontal wall assert that you've seen an odd number of corners
        let mut area: usize = 0;
        for r in self.minr..=self.maxr {
            let mut inside = false;
            let mut last_corner_opt: Option<Corner> = None;
            for c in self.minc..=self.maxc {
                let this_loc = self.get(Loc(r, c));
                inside = match this_loc {
                    Space::Dot => inside,                   // leave it unchanged
                    Space::Flat(Flat::Vertical) => !inside, // flip it
                    Space::Flat(Flat::Horizontal) => {
                        debug_assert!(matches!(last_corner_opt, Some(_)));
                        inside
                    }
                    Space::Corner(this_corner) => match last_corner_opt {
                        // entering a horizontal run
                        None => {
                            last_corner_opt = Some(this_corner);
                            inside
                        }
                        // exiting a horizontal run
                        Some(last_corner) => {
                            let x = match (last_corner, this_corner) {
                                (Corner::TopRight, Corner::BottomLeft) => !inside,
                                (Corner::TopRight, Corner::TopLeft) => inside,
                                (Corner::BottomRight, Corner::TopLeft) => !inside,
                                (Corner::BottomRight, Corner::BottomLeft) => inside,
                                _ => panic!(
                                    "oh no {:?}",
                                    (last_corner, this_corner, r - self.minr, c - self.minc)
                                ),
                            };
                            last_corner_opt = None;
                            x
                        }
                    },
                };

                if matches!(this_loc, Space::Corner(_) | Space::Flat(_)) || inside {
                    print!("#");
                    area += 1;
                } else {
                    print!(".");
                }
            }
            println!();
        }
        area
    }

    fn print_shell(&self) {
        for r in self.minr..=self.maxr {
            for c in self.minc..=self.maxc {
                if self.locations.contains(&Loc(r, c)) {
                    print!("#");
                } else {
                    print!(".");
                }
            }
            println!();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Loc(isize, isize);

impl Loc {
    fn mv(&self, dir: Direction) -> Loc {
        let Loc(r, c) = *self;
        match dir {
            Direction::Up => Loc(r - 1, c),
            Direction::Left => Loc(r, c - 1),
            Direction::Right => Loc(r, c + 1),
            Direction::Down => Loc(r + 1, c),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Left,
    Right,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Instruction {
    direction: Direction,
    num: usize,
}

fn parse(content: &str) -> Result<Vec<Instruction>, Box<dyn Error>> {
    let instructions: Vec<_> = content
        .lines()
        .map(|line| {
            let parts: Vec<_> = line.split_ascii_whitespace().collect();
            assert!(parts.len() == 3);
            let (direction, num) = (parts[0], parts[1]);
            let direction = match direction {
                "R" => Direction::Right,
                "L" => Direction::Left,
                "U" => Direction::Up,
                "D" => Direction::Down,
                _ => panic!("oh no"),
            };
            let num: usize = num.parse().unwrap();
            Instruction { direction, num }
        })
        .collect();
    Ok(instructions)
}

fn main() -> Result<(), Box<dyn Error>> {
    // parse input -> instructions
    let content = fs::read_to_string("src/d18/input")?;

    // parse to grid
    // in grid:
    //      identify each space as Corner(corner) or Wall(vert|horizontal)
    let grid = Grid::parse(&content)?;

    grid.print_shell();

    println!();
    println!();
    println!();
    let area = grid.get_area();

    println!("part 1 {}", area);

    // setup box

    // print_shell(&locations, (minr, maxr, minc, maxc));

    // sweep line Right -> Left (switch state)

    Ok(())
}
