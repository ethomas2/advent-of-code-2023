use std::collections::HashMap;
use std::error::Error;
use std::fs;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum Space {
    Hash,
    Unknown,
    Dot,
}

fn parse_line(line: &str) -> Result<(Vec<Space>, Vec<usize>), Box<dyn Error>> {
    let splitline: Vec<_> = line.split(' ').collect();
    let (first, second) = (splitline[0], splitline[1]);
    let spaces: Vec<_> = first
        .chars()
        .map(|ch| match ch {
            '#' => Ok(Space::Hash),
            '?' => Ok(Space::Unknown),
            '.' => Ok(Space::Dot),
            _ => Err("oh no".to_owned()),
        })
        .collect::<Result<_, _>>()?;

    let numbers: Vec<_> = second
        .split(',')
        .map(|x| x.parse::<usize>())
        .collect::<Result<_, _>>()?;

    Ok((spaces, numbers))
}

macro_rules! return_and_insert {
    ($memo:expr, $spaces:expr, $numbers:expr, $val:expr) => {{
        let val = $val;
        $memo.insert(($spaces.to_vec(), $numbers.to_vec()), val);
        return val;
    }};
}

fn num_solutions(
    spaces: &[Space],
    numbers: &[usize],
    memo: &mut HashMap<(Vec<Space>, Vec<usize>), usize>,
) -> usize {
    if let Some(&v) = memo.get(&(spaces.to_vec(), numbers.to_vec())) {
        return v;
    }

    if spaces.is_empty() {
        return_and_insert!(memo, spaces, numbers, numbers.is_empty() as usize);
    }

    if numbers.is_empty() {
        let all_dot_or_unknown = spaces
            .iter()
            .all(|s| matches!(s, Space::Dot | Space::Unknown));
        return_and_insert!(memo, spaces, numbers, all_dot_or_unknown as usize);
    }

    let numbers_sum: usize = numbers.iter().sum::<usize>() + (numbers.len().saturating_sub(1));
    if spaces.len() < numbers_sum {
        return_and_insert!(memo, spaces, numbers, 0);
    }

    match spaces[0] {
        Space::Dot => {
            let v = num_solutions(&spaces[1..], numbers, memo);
            return_and_insert!(memo, spaces, numbers, v);
        }
        Space::Hash => {
            let all_hash_or_unknown = spaces[..numbers[0]]
                .iter()
                .all(|&space| matches!(space, Space::Unknown | Space::Hash));
            if !all_hash_or_unknown {
                return_and_insert!(memo, spaces, numbers, 0);
            }
            if spaces.len() == numbers[0] {
                return_and_insert!(memo, spaces, numbers, 1);
            }
            debug_assert!(spaces.len() > numbers[0]);
            if spaces[numbers[0]] == Space::Hash {
                return_and_insert!(memo, spaces, numbers, 0);
            }
            let ans = num_solutions(&spaces[(numbers[0] + 1)..], &numbers[1..], memo);
            return_and_insert!(memo, spaces, numbers, ans);
        }
        Space::Unknown => {
            let x1: usize = {
                let mut spaces_clone = spaces.to_vec();
                spaces_clone[0] = Space::Hash;
                num_solutions(&spaces_clone, numbers, memo)
            };

            let x2: usize = {
                let mut spaces_clone = spaces.to_vec();
                spaces_clone[0] = Space::Dot;
                num_solutions(&spaces_clone, numbers, memo)
            };
            return_and_insert!(memo, spaces, numbers, x1 + x2);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test() {
        let mut memo: HashMap<(Vec<Space>, Vec<usize>), usize> = HashMap::new();
        let (spaces, _) = parse_line("#..# 1,1").unwrap();
        assert_eq!(num_solutions(&spaces, &[1, 1], &mut memo), 1);

        let (spaces, _) = parse_line("####.##.# 1,1").unwrap();
        assert_eq!(num_solutions(&spaces, &[4, 2, 1], &mut memo), 1);

        let (spaces, _) = parse_line("####.##.# 1,1").unwrap();
        assert_eq!(num_solutions(&spaces, &[3, 2, 1], &mut memo), 0);

        let (spaces, numbers) = parse_line("#? 1,1").unwrap();
        assert_eq!(num_solutions(&spaces, &numbers, &mut memo), 0);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d12/input")?;
    // TODO: use smallvec everywhere
    let mut memo: HashMap<(Vec<Space>, Vec<usize>), usize> = HashMap::new();
    let mut parsed_lines: Vec<_> = content
        .lines()
        .map(|line| parse_line(line))
        .collect::<Result<_, _>>()?;
    let s: usize = parsed_lines
        .iter_mut()
        .map(|(spaces, numbers)| num_solutions(spaces, numbers, &mut memo))
        .sum();
    println!("s1 {}", s);

    let mut i = 1;
    let mut memo: HashMap<(Vec<Space>, Vec<usize>), usize> = HashMap::new();
    let s2: usize = parsed_lines
        .iter()
        .map(|(spaces, numbers)| {
            let mut new_spaces: Vec<Space> = Vec::with_capacity(spaces.len() * 5 + 4);
            for _ in 0..4 {
                new_spaces.extend(spaces.iter().cloned());
                new_spaces.push(Space::Unknown);
            }
            new_spaces.extend(spaces.iter().cloned());

            let mut new_numbers: Vec<usize> = Vec::with_capacity(5 * numbers.len());
            for _ in 0..5 {
                new_numbers.extend(numbers.iter());
            }
            i += 1;
            (new_spaces, new_numbers)
        })
        .map(|(mut spaces, numbers)| {
            let nsolutions = num_solutions(&mut spaces, &numbers, &mut memo);
            nsolutions
        })
        .sum();

    println!("s2 {}", s2);
    Ok(())
}
