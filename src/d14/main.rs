use std::error::Error;
use std::fmt::{Display, Write};
use std::fs;

struct Grid<T> {
    width: usize,
    height: usize,
    grid: Vec<T>,
}

impl<T> Grid<T> {
    fn get(&self, (r, c): (usize, usize)) -> &T {
        &self.grid[r * self.width + c]
    }

    fn get_mut(&mut self, (r, c): (usize, usize)) -> &mut T {
        &mut self.grid[r * self.width + c]
    }
}

impl<T> Display for Grid<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in 0..self.height {
            for c in 0..self.width {
                write!(f, "{}", self.get((r, c)))?;
            }
            writeln!(f, "")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

fn slide_north(grid: &mut Grid<Space>) {
    for r in 1..grid.height {
        for c in 0..grid.width {
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
    #[test]
    fn test_slide_north() {
        let content = "\
O....#....
O.OO#....#
.....##...
OO.#O....O
.O.....O#.
O.#..O.#.#
..O..#O..O
.......O..
#....###..
#OO..#....";
        let mut grid = parse(content);
        slide_north(&mut grid);
        let expected = "\
OOOO.#.O..
OO..#....#
OO..O##..O
O..#.OO...
........#.
..#....#.#
..O..#.O.O
..O.......
#....###..
#....#....";
        assert_eq!(format!("{}", grid).trim(), expected.trim());
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d14/input")?;
    let mut grid = parse(&content);

    slide_north(&mut grid);
    let s1 = {
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
    };
    println!("s1 {}", s1);

    Ok(())
}
