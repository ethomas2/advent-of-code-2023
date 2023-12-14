use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::fs;
use std::hash::{Hash, Hasher};

#[derive(Clone, PartialEq, Eq)]
struct Grid<T> {
    width: usize,
    height: usize,
    grid: Vec<T>,
}

impl<T: Hash> Hash for Grid<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.width.hash(state);
        self.height.hash(state);
        for item in &self.grid {
            item.hash(state);
        }
    }
}

impl<T> Grid<T> {
    fn grid_from_streams<OuterIter, InnerIter>(stream: OuterIter) -> Grid<T>
    where
        OuterIter: Iterator<Item = InnerIter>,
        InnerIter: Iterator<Item = T>,
    {
        let mut grid = Vec::new();
        let mut height = 0;
        let mut width = 0;
        for iter in stream {
            width = 0;
            height += 1;
            for item in iter {
                width += 1;
                grid.push(item);
            }
        }
        Grid {
            grid,
            width,
            height,
        }
    }
    fn get(&self, (r, c): (usize, usize)) -> &T {
        &self.grid[r * self.width + c]
    }

    fn get_mut(&mut self, (r, c): (usize, usize)) -> &mut T {
        &mut self.grid[r * self.width + c]
    }

    fn rotate_mut(&mut self, rot: Rot) -> GridViewMutImpl<T> {
        GridViewMutImpl { grid: self, rot }
    }
}

enum Rot {
    Rot0,
    Rot90,
    Rot180,
    Rot270,
}
struct GridViewMutImpl<'a, T> {
    // TODO: make this an Rc<RefCell instead of a mutable pointer to the grid
    grid: &'a mut Grid<T>,
    rot: Rot,
}

impl<'a, T> GridViewMut for GridViewMutImpl<'a, T> {
    type Item = T;

    fn get(&self, (r, c): (usize, usize)) -> &T {
        let (rnew, cnew) = match self.rot {
            Rot::Rot0 => (r, c),
            Rot::Rot90 => (c, self.grid.width - 1 - r),
            Rot::Rot180 => (self.grid.height - 1 - r, self.grid.width - 1 - c),
            Rot::Rot270 => (self.grid.height - 1 - c, r),
        };
        &self.grid.grid[self.grid.width * rnew + cnew]
    }
    fn get_mut(&mut self, (r, c): (usize, usize)) -> &mut T {
        let (rnew, cnew) = match self.rot {
            Rot::Rot0 => (r, c),
            Rot::Rot90 => (c, self.grid.width - 1 - r),
            Rot::Rot180 => (self.grid.height - 1 - r, self.grid.width - 1 - c),
            Rot::Rot270 => (self.grid.height - 1 - c, r),
        };
        &mut self.grid.grid[self.grid.width * rnew + cnew]
    }

    fn height(&self) -> usize {
        match self.rot {
            Rot::Rot0 | Rot::Rot180 => self.grid.height,
            Rot::Rot90 | Rot::Rot270 => self.grid.width,
        }
    }
    fn width(&self) -> usize {
        match self.rot {
            Rot::Rot0 | Rot::Rot180 => self.grid.width,
            Rot::Rot90 | Rot::Rot270 => self.grid.height,
        }
    }
}

/// TODO: explain this more
trait GridViewMut {
    type Item;

    fn get(&self, loc: (usize, usize)) -> &Self::Item;
    fn get_mut(&mut self, loc: (usize, usize)) -> &mut Self::Item;
    fn height(&self) -> usize;
    fn width(&self) -> usize;
}

impl<T> GridViewMut for Grid<T> {
    type Item = T;

    fn get(&self, (r, c): (usize, usize)) -> &T {
        &self.grid[r * self.width + c]
    }

    fn get_mut(&mut self, (r, c): (usize, usize)) -> &mut T {
        &mut self.grid[r * self.width + c]
    }

    fn height(&self) -> usize {
        self.height
    }

    fn width(&self) -> usize {
        self.width
    }
}

impl<T: Display> Display for Grid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in 0..self.height() {
            for c in 0..self.width() {
                write!(f, "{}", self.get((r, c)))?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}

impl<'a, T> Display for GridViewMutImpl<'a, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in 0..self.height() {
            for c in 0..self.width() {
                write!(f, "{}", self.get((r, c)))?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Space {
    Dot,
    Round,
    Hash,
}

impl Display for Space {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Space::Dot => write!(f, ".")?,
            Space::Round => write!(f, "O")?,
            Space::Hash => write!(f, "#")?,
        }
        Ok(())
    }
}

fn slide_north<T: GridViewMut<Item = Space>>(grid: &mut T) {
    for r in 1..grid.height() {
        for c in 0..grid.width() {
            if *grid.get((r, c)) == Space::Round {
                let row_to_move_to = {
                    let mut row_to_move_to = r;
                    while row_to_move_to > 0 && *grid.get((row_to_move_to - 1, c)) == Space::Dot {
                        row_to_move_to -= 1;
                    }
                    row_to_move_to
                };
                if row_to_move_to != r {
                    *grid.get_mut((row_to_move_to, c)) = Space::Round;
                    *grid.get_mut((r, c)) = Space::Dot;
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Direction {
    North,
    East,
    South,
    West,
}

fn slide(grid: &mut Grid<Space>, dir: Direction) {
    let rotation = match dir {
        Direction::North => Rot::Rot0,
        Direction::South => Rot::Rot180,
        Direction::West => Rot::Rot270,
        Direction::East => Rot::Rot90,
    };
    let mut rotated = grid.rotate_mut(rotation);
    slide_north(&mut rotated);
}

fn parse(content: &str) -> Grid<Space> {
    let grid: Vec<_> = content
        .lines()
        .flat_map(|line| line.chars())
        .map(|ch| match ch {
            'O' => Space::Round,
            '.' => Space::Dot,
            '#' => Space::Hash,
            _ => panic!("unexpected character {}", ch),
        })
        .collect();
    let width = content.lines().nth(0).unwrap().len();
    let height = content.lines().count();
    Grid {
        grid,
        width,
        height,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;
    #[test]
    fn test_slide_north() {
        let content = indoc! { "
            O....#....
            O.OO#....#
            .....##...
            OO.#O....O
            .O.....O#.
            O.#..O.#.#
            ..O..#O..O
            .......O..
            #....###..
            #OO..#....
        "};
        let mut grid = parse(content);
        slide_north(&mut grid);
        let expected = indoc! {"
            OOOO.#.O..
            OO..#....#
            OO..O##..O
            O..#.OO...
            ........#.
            ..#....#.#
            ..O..#.O.O
            ..O.......
            #....###..
            #....#....
        "};
        assert_eq!(format!("{}", grid).trim(), expected.trim());
    }

    #[test]
    fn test_rotation() {
        let iter = indoc! {"
            abc
            def
        " }
        .lines()
        .map(|line| line.chars());
        let mut grid = Grid::grid_from_streams(iter);

        assert_eq!(
            format!("{}", grid.rotate_mut(Rot::Rot90)),
            indoc! {"
            cf
            be
            ad
        "}
        );

        assert_eq!(
            format!("{}", grid.rotate_mut(Rot::Rot180)),
            indoc! {"
            fed
            cba
        "}
        );

        assert_eq!(
            format!("{}", grid.rotate_mut(Rot::Rot270)),
            indoc! {"
            da
            eb
            fc
        "}
        );
    }
}

fn get_load(grid: &Grid<Space>) -> usize {
    let mut s1 = 0;
    for r in 0..grid.height {
        for c in 0..grid.width {
            if *grid.get((r, c)) == Space::Round {
                let value = grid.height - r;
                s1 += value;
            }
        }
    }
    s1
}

fn main() -> Result<(), Box<dyn Error>> {
    // part 1
    {
        let content = fs::read_to_string("src/d14/input")?;
        let mut grid = parse(&content);
        slide_north(&mut grid);
        println!("s1 {}", get_load(&grid));
    }

    // part 2
    {
        let content = fs::read_to_string("src/d14/input")?;
        let mut grid = parse(&content);
        let cycle = |grid: &mut Grid<Space>| {
            slide(grid, Direction::North);
            slide(grid, Direction::West);
            slide(grid, Direction::South);
            slide(grid, Direction::East);
        };
        // i: first time we saw this repeated grid
        // j: second time we saw this repeated grid
        // grid: the repeated grid
        let (i, j, mut grid) = 'a: {
            let mut visited: HashMap<Grid<Space>, usize> = HashMap::new();
            for j in 1.. {
                cycle(&mut grid);
                if let Some(&i) = visited.get(&grid) {
                    break 'a (i, j, grid);
                }
                visited.insert(grid.clone(), j);
            }
            unreachable!();
        };
        let billionth_grid = {
            let B = 1_000_000_000;
            let epsilon = (B - j) % (j - i);
            let k = B - epsilon; // k is the last time we see this repeated grid
                                 // before 1 billion
            println!("i {} j {} epsilon {} k {}", i, j, epsilon, k);
            for _ in 0..(B - k) {
                cycle(&mut grid);
            }
            grid
        };
        println!("s2 {}", get_load(&billionth_grid));
    }

    Ok(())
}
