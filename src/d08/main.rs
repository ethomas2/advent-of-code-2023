use std::collections::HashMap;
use std::error::Error;
use std::fs;

use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, line_ending, space0},
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
            alpha1,
            space0,
            tag("="),
            space0,
            tag("("),
            alpha1,
            tag(","),
            space0,
            alpha1,
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

    let mut i = 0;
    let mut current = "AAA";
    'outer: loop {
        for x in instructions.chars() {
            if current == "ZZZ" {
                break 'outer;
            }
            match x {
                'L' => current = map.get(current).unwrap().0,

                'R' => current = map.get(current).unwrap().1,
                _ => panic!("unexpected character {}", x),
            }
            i += 1;
        }
    }
    println!("{}", i);

    Ok(())
}
