use itertools::Itertools;
use std::error::Error;
use std::fs;

#[derive(Debug, PartialEq, Eq, Clone)]
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

fn num_solutions(spaces: &mut Vec<Space>, numbers: &Vec<usize>) -> usize {
    let known_prefix = {
        // longest prefix that doesn't contain ?
        // TODO: slice by first non-dot?
        let first_unknown_idx =
            spaces
                .iter()
                .enumerate()
                .find_map(|(i, s)| if s == &Space::Unknown { Some(i) } else { None });
        match first_unknown_idx {
            Some(idx) => &spaces[0..idx],
            None => &spaces[0..spaces.len()],
        }
    };

    let is_possible = 'outer: {
        // if the given known_prefix is possible given the number set. I.e.
        // ####... 1,2 is not possible
        let mut numbers_idx = 0;
        let mut hash_run = 0;
        for (i, x) in known_prefix.iter().enumerate() {
            match x {
                &Space::Hash => {
                    hash_run += 1;
                    if numbers_idx >= numbers.len() || hash_run > numbers[numbers_idx] {
                        // dbg!("first break");
                        break 'outer false;
                    }
                }
                &Space::Dot => {
                    let is_transition = i > 0 && known_prefix[i - 1] == Space::Hash;
                    // increment numbers_idx when you transition
                    if is_transition {
                        if hash_run != numbers[numbers_idx] {
                            // dbg!("second break", i, numbers_idx);
                            break 'outer false;
                        }
                        numbers_idx += 1;
                    }
                    hash_run = 0;
                }
                &Space::Unknown => panic!("logic error"),
            }
        }
        true
    };

    if !is_possible {
        return 0;
    }

    if known_prefix.len() == spaces.len() {
        // check if everything exactly matches
        let completed_number_set: Vec<_> = spaces
            .iter()
            .group_by(|&x| x.clone())
            .into_iter()
            .filter(|(k, _)| k == &Space::Hash)
            .map(|(_, x)| x.count())
            .collect();
        if &completed_number_set != numbers {
            return 0;
        } else {
            return 1;
        }
    }

    let n_solutions = {
        let i = known_prefix.len();
        debug_assert_eq!(spaces[i], Space::Unknown);
        spaces[i] = Space::Hash;
        let n1 = num_solutions(spaces, numbers);
        spaces[i] = Space::Dot;
        let n2 = num_solutions(spaces, numbers);
        spaces[i] = Space::Unknown;
        n1 + n2
    };

    n_solutions
}

fn main() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d12/input")?;
    let mut parsed_lines: Vec<_> = content
        .lines()
        .map(|line| parse_line(line))
        .collect::<Result<_, _>>()?;
    let s: usize = parsed_lines
        .iter_mut()
        .map(|(spaces, numbers)| num_solutions(spaces, numbers))
        .sum();
    println!("s {}", s);
    Ok(())
}
