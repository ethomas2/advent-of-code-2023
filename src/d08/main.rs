use std::collections::HashMap;
use std::error::Error;
use std::fs;

use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, line_ending, space0},
    combinator::{all_consuming, map},
    multi::{count, many0, separated_list1},
    sequence::tuple,
    IResult,
};

/// src -> (left, right)
type Map<'a> = HashMap<&'a str, (&'a str, &'a str)>;

fn parse(input: &str) -> IResult<&str, (&str, Map)> {
    let (input, instructions) = alpha1(input)?;

    let (input, _) = many0(line_ending)(input)?;

    let parse_line = map(
        tuple((
            alphanumeric1,
            space0,
            tag("="),
            space0,
            tag("("),
            alphanumeric1,
            tag(","),
            space0,
            alphanumeric1,
            tag(")"),
        )),
        |(src, _, _, _, _, left, _, _, right, _)| (src, (left, right)),
    );

    let (input, vec) = separated_list1(line_ending, parse_line)(input)?;
    let (input, _) = line_ending(input)?;

    let map: Map = vec.into_iter().collect();

    Ok((input, (instructions, map)))
}

fn main() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d08/input")?;
    let (_, (instructions, map)) = all_consuming(parse)(&content).unwrap();

    let mut iterators = Vec::new();
    for start in map.keys() {
        if start.chars().nth(2).unwrap() == 'A' {
            let mut current = *start;
            let mut instruction_iterator = instructions.chars().cycle();
            let map = &map;
            let iter = std::iter::from_fn(move || {
                let instruction = instruction_iterator.next().unwrap();
                match instruction {
                    'L' => current = &map.get(current).unwrap().0,
                    'R' => current = &map.get(current).unwrap().1,
                    _ => panic!("unexpected character {}", instruction),
                };
                Some(current)
            });
            // TODO: why don't i have to clone current
            iterators.push(std::iter::once(current).chain(iter));
        }
    }

    // think it might be slow now becaues i'm doing a bunch of memcopys instaed of just all
    let mut i: usize = 0;
    let mut state = Vec::new();
    loop {
        state.truncate(0);
        state.extend(iterators.iter_mut().map(|iter| iter.next().unwrap()));

        i += 1;
        if i % 50_000_000 == 0 {
            println!("{}", i);
        }

        if state.iter().all(|x| x.chars().nth(2).unwrap() == 'Z') {
            break;
        }
    }

    println!("Answer :: {}", i);

    Ok(())
}
