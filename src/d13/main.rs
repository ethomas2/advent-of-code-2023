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

fn find_first_vertical_reflection<T: Eq>(vec: &Grid<T>) -> Option<usize> {
    let width = vec[0].len();
    (1..width).find(|&col| vec.iter().all(|v| check_is_reflection_line(v, col)))
}

fn find_first_horizontal_reflection<T: Eq + Clone>(vec: &Grid<T>) -> Option<usize> {
    find_first_vertical_reflection(&transpose(&vec))
}

fn transpose<T: Clone>(v: &Grid<T>) -> Grid<T> {
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
    let grids = parse(&content);
    let mut sum: usize = 0;
    for grid in grids {
        if let Some(reflection_line) = find_first_vertical_reflection(&grid) {
            sum += reflection_line;
        } else if let Some(reflection_line) = find_first_horizontal_reflection(&grid) {
            sum += 100 * reflection_line;
        } else {
            panic!("oh no");
        }
    }
    println!("{:?}", sum);
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
