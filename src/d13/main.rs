use itertools::iproduct;
use std::error::Error;
use std::fs;

fn check_is_reflection_line<T: Eq>(v: &Vec<T>, col: usize) -> bool {
    let mut diff = 0;
    loop {
        if diff + 1 > col || col + diff >= v.len() {
            return true;
        }
        if v[col - diff - 1] != v[col + diff] {
            return false;
        }
        diff += 1;
    }
    return true;
}

fn find_reflection_lines<T: Eq>(v: &Vec<T>) -> impl Iterator<Item = usize> + '_ {
    (1..v.len()).filter(|&col| check_is_reflection_line(v, col))
}

fn find_vertical_reflection_lines<T: Eq>(vec: &Grid<T>) -> Vec<usize> {
    let width = vec[0].len();
    (1..width)
        .filter(|&col| vec.iter().all(|v| check_is_reflection_line(v, col)))
        .collect()
}

fn find_first_vertical_reflection<T: Eq>(vec: &Grid<T>) -> Option<usize> {
    find_vertical_reflection_lines(vec).first().copied()
}

fn find_horizontal_reflection_lines<T: Eq + Clone>(vec: &Grid<T>) -> Vec<usize> {
    find_vertical_reflection_lines(&transpose(&vec))
}

fn find_first_horizontal_reflection<T: Eq + Clone>(vec: &Grid<T>) -> Option<usize> {
    find_horizontal_reflection_lines(vec).first().copied()
}

fn transpose<T: Clone>(v: &Grid<T>) -> Grid<T> {
    // TODO: instead of allocating a new grid return a view on top of the current grid
    let height = v.len();
    let width = v[0].len();
    let mut ret: Grid<T> = (0..width).map(|_| Vec::with_capacity(height)).collect();
    for r in 0..height {
        for c in 0..width {
            ret[c].push(v[r][c].clone());
        }
    }
    ret
}

type Grid<T> = Vec<Vec<T>>;

fn parse(content: &str) -> impl Iterator<Item = Grid<char>> + '_ {
    let x = content.split("\n\n").map(|chunk| {
        chunk
            .trim()
            .lines()
            .map(|line| line.chars().collect())
            .collect()
    });
    x
}

fn main() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d13/input")?;
    let grids: Vec<_> = parse(&content).collect();

    // part 1
    let mut sum1: usize = 0;
    let mut original_reflection_lines = Vec::new();
    for grid in grids.iter() {
        if let Some(reflection_line) = find_first_vertical_reflection(&grid) {
            sum1 += reflection_line;
            original_reflection_lines.push(('v', reflection_line));
        } else if let Some(reflection_line) = find_first_horizontal_reflection(&grid) {
            sum1 += 100 * reflection_line;
            original_reflection_lines.push(('h', reflection_line));
        } else {
            panic!("oh no");
        }
    }
    println!("part 1 {:?}", sum1);

    // part 2
    let mut sum2: usize = 0;
    for (grid_idx, grid) in grids.iter().enumerate() {
        let (width, height) = (grid[0].len(), grid.len());
        let (ch, l) = iproduct!(0..height, 0..width)
            .flat_map(|(r, c)| {
                let mut new_grid = grid.clone();
                new_grid[r][c] = if grid[r][c] == '#' { '.' } else { '#' };

                let new_veritical_reflection_lines = find_vertical_reflection_lines(&new_grid)
                    .into_iter()
                    .map(|line| ('v', line));
                let new_horizontal_reflection_lines = find_horizontal_reflection_lines(&new_grid)
                    .into_iter()
                    .map(|line| ('h', line));

                new_veritical_reflection_lines.chain(new_horizontal_reflection_lines)
            })
            .find(|&x| x != original_reflection_lines[grid_idx])
            .unwrap();

        if ch == 'v' {
            sum2 += l;
        } else {
            sum2 += 100 * l;
        }
    }
    println!("part 2 {:?}", sum2);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_stuff() {
        assert_eq!(
            find_reflection_lines(&"###".chars().collect()).collect::<Vec<_>>(),
            vec![1, 2]
        );

        assert_eq!(
            find_reflection_lines(&"#.#".chars().collect()).collect::<Vec<_>>(),
            vec![]
        );

        assert_eq!(
            find_reflection_lines(&"#..#".chars().collect()).collect::<Vec<_>>(),
            vec![2]
        );

        assert_eq!(
            find_reflection_lines(&"....##..##...".chars().collect()).collect::<Vec<_>>(),
            vec![1, 2, 7, 12]
        );
    }
}
