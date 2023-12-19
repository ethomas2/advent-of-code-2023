use std::error::Error;
use std::fs;
use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Point(isize, isize);

impl Point {
    fn mv(&self, dir: Direction, n: isize) -> Self {
        let Self(r, c) = *self;
        match dir {
            Direction::Up => Self(r - n, c),
            Direction::Left => Self(r, c - n),
            Direction::Right => Self(r, c + n),
            Direction::Down => Self(r + n, c),
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

impl Direction {
    fn turn_type(&self, nextdir: &Self) -> Option<TurnType> {
        let turn_type = match (self, nextdir) {
            (Self::Up, Self::Left)
            | (Self::Right, Self::Up)
            | (Self::Down, Self::Right)
            | (Self::Left, Self::Down) => TurnType::CounterClockwise,
            (Self::Up, Self::Right)
            | (Self::Left, Self::Up)
            | (Self::Down, Self::Left)
            | (Self::Right, Self::Down) => TurnType::Clockwise,
            _ => return None,
        };
        Some(turn_type)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum TurnType {
    Clockwise,
    CounterClockwise,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Instruction {
    direction: Direction,
    num: usize,
}

fn parse_instructions_part1(content: &str) -> Result<Vec<Instruction>, Box<dyn Error>> {
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
                _ => panic!("unexpected character {}", direction),
            };
            let num: usize = num.parse().unwrap();
            Instruction { direction, num }
        })
        .collect();
    Ok(instructions)
}

fn parse_instructions_part2(content: &str) -> Result<Vec<Instruction>, Box<dyn Error>> {
    let instructions: Vec<_> = content
        .lines()
        .map(|line| {
            let parts: Vec<_> = line.split_ascii_whitespace().collect();
            assert!(parts.len() == 3);
            let x = parts[2];
            let num = usize::from_str_radix(&x[2..7], 16).unwrap();
            let dir = x[7..8].parse::<usize>().unwrap();
            let direction = match dir {
                0 => Direction::Right,
                1 => Direction::Down,
                2 => Direction::Left,
                3 => Direction::Up,
                _ => panic!("unexpected last character {}", dir),
            };
            Instruction { direction, num }
        })
        .collect();
    Ok(instructions)
}

fn points_for_hash(upperleft: Point) -> [Point; 4] {
    // let mut avec = ArrayVec::new();
    let Point(r, c) = upperleft;
    [
        Point(r, c),
        Point(r + 1, c),
        Point(r, c + 1),
        Point(r + 1, c + 1),
    ]
}

/// Given a list of Instructions, compute the "bounding points" for the polygon carved out by these
/// instructions. I.e. the set of points necessary to compute shoelace theorem
fn get_bounding_points(instructions: &Vec<Instruction>) -> Vec<Point> {
    // Terms:
    //      hashpoint: The upper left point for the "bounding hash". Where a "bounding hash" is a
    //      hash at which the diagram turns a corner. In the diagram below the hashpoints are
    //      (0, 0), (0, 6), (5, 6), (5, 4), (7, 4), (7, 6) ...
    //
    //      bounding_point: A point that "bounds" the diagram. A bounding point always touches a
    //      bounding hash, but is not necessarily the upper left point of the bounding hash (and
    //      therefore is not necessarily a hashpoint). In the diagram below the bounding points are
    //      (0, 0), (0, 7), (6, 7), (6, 5), (7, 5), (7, 7)
    //
    //  Example Diagram:
    //
    //          #######
    //          #.....#
    //          ###...#
    //          ..#...#
    //          ..#...#
    //          ###.###
    //          #...#..
    //          ##..###
    //          .#....#
    //          .######
    //
    // Algorithm:
    //     let hashpoint = (0, 0)
    //     let polyloc = (0, 0)
    //     for instruction in instructios
    //          move hashpoint in the current direction n times
    //          let turn_type = "the direction you will turn to follow the next instruction"
    //          if turn_type == clockwise:
    //              move bounding_point until it touches the "final" possible point that touches
    //              this bounding hash
    //          else if rotating counter clockwise:
    //              move bounding_point until it touches the *first* possible point that touches
    //              this bounding hash
    //          record
    let mut hashpoint = Point(0, 0);
    let mut bounding_point = Point(0, 0);
    let bounding_points = {
        let mut bounding_points = Vec::new();
        for (i, &instruction) in instructions.iter().enumerate() {
            let Instruction { direction, num } = instruction;
            // move hashpoint in the current direction n times
            hashpoint = hashpoint.mv(direction, num.try_into().unwrap());

            // compute turn_type (the direction you will be turning next)
            let Instruction {
                direction: nextdir, ..
            } = instructions[(i + 1) % instructions.len()];
            let turn_type = direction.turn_type(&nextdir).unwrap();

            // update bounding point. You must at least walk until it touches the first possible
            // point that touches this bounding hash
            let hash_points = points_for_hash(hashpoint);
            while !hash_points.contains(&bounding_point) {
                bounding_point = bounding_point.mv(direction, 1);
            }
            // and go one more if it's turning away from you. This will still touch the bounding
            // hash and will be the "final" possible point that touches this bounding hash
            if matches!(turn_type, TurnType::Clockwise) {
                bounding_point = bounding_point.mv(direction, 1);
                debug_assert!(hash_points.contains(&bounding_point));
            }
            bounding_points.push(bounding_point);
        }
        bounding_points
    };
    bounding_points
}

/// compute a - b mod n. Rust's % is _remainder_, not _modululs_ (so it doesn't work on negative
/// numbers).  -1 % 5 = -1 in rust. So do this hacky workaround
fn sub_mod_n(a: usize, b: usize, n: usize) -> usize {
    let a: isize = a.try_into().unwrap();
    let b: isize = b.try_into().unwrap();
    let n: isize = n.try_into().unwrap();
    let x = a - b;
    let ans = ((x % n) + n) % n;
    ans.try_into().unwrap()
}

/// Compute shoelace theorem for the given points
/// https://artofproblemsolving.com/wiki/index.php/Shoelace_Theorem
fn shoelace(points: &Vec<Point>) -> usize {
    let mut area = 0;
    for (i, p) in points.iter().enumerate() {
        let Point(r, _) = p;
        let Point(_, c2) = points[(i + 1) % points.len()];
        let Point(_, c0) = points[sub_mod_n(i, 1, points.len())];
        area += (r * c2) - (r * c0);
    }
    area = area.abs() / 2;
    area.try_into().unwrap()
}

fn main() -> Result<(), Box<dyn Error>> {
    // parse input -> instructions
    let content = fs::read_to_string("src/d18/input")?;

    // part 1
    let instructions = parse_instructions_part1(&content)?;
    let points = get_bounding_points(&instructions);
    let area = shoelace(&points);
    println!("part 1 {:?}", area);

    // part 2
    let instructions = parse_instructions_part2(&content)?;
    let points = get_bounding_points(&instructions);
    let area = shoelace(&points);
    println!("part 2 {:?}", area);

    Ok(())
}
