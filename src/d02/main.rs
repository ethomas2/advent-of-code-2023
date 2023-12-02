use std::error::Error;
use std::fs;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{space0, space1},
    combinator::{all_consuming, opt},
    multi::many1,
    sequence::{preceded, terminated, tuple},
    IResult,
};

#[derive(Debug, Default, PartialEq, Eq)]
struct CubeSet {
    red: u32,
    green: u32,
    blue: u32,
}

#[derive(Debug, PartialEq, Eq)]
struct Game {
    gameid: u32,
    cube_sets: Vec<CubeSet>,
}

/// Parse a single cube set. ie
///  3 blue, 4 red
///     or
///  3 blue, 4 red, 5 green
fn parse_cube_set(input: &str) -> IResult<&str, CubeSet> {
    let (input, cube_tuples) = many1(|input| -> IResult<&str, (&str, u32)> {
        let (input, (n, _, color_name, _, _)) = tuple((
            nom::character::complete::u32,
            space1,
            alt((tag("red"), tag("blue"), tag("green"))),
            opt(tag(",")),
            space0,
        ))(input)?;

        Ok((input, (color_name, n)))
    })(input)?;

    let mut cubeset: CubeSet = Default::default();
    for (cube_name, n) in cube_tuples {
        match cube_name {
            "red" => cubeset.red = n,
            "green" => cubeset.green = n,
            "blue" => cubeset.blue = n,
            _ => unreachable!(),
        }
    }

    Ok((input, cubeset))
}

/// Parse a line, eg
///         Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue
/// becomes
///         Game { gameid: 1, cube_sets: [CubeSet { red: 4, green: 0, blue: 3 }, CubeSet { red: 1, green: 2, blue: 6 }] }
fn parse_line(input: &str) -> IResult<&str, Game> {
    let (input, (_, _, gameid, _, _)) = tuple((
        tag("Game"),
        space1,
        nom::character::complete::u32,
        tag(":"),
        space1,
    ))(input)?;

    let (input, cube_sets) = many1(|input| -> IResult<&str, _> {
        let (input, (cubeset, _, _)) = tuple((parse_cube_set, opt(tag(";")), space0))(input)?;
        Ok((input, cubeset))
    })(input)?;
    Ok((input, Game { gameid, cube_sets }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test1() {
        let input = "Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green";
        let (_, game) = all_consuming(parse_line)(input).unwrap();
        assert_eq!(
            game,
            Game {
                gameid: 1,
                cube_sets: vec![
                    CubeSet {
                        red: 4,
                        green: 0,
                        blue: 3,
                    },
                    CubeSet {
                        red: 1,
                        green: 2,
                        blue: 6,
                    },
                    CubeSet {
                        red: 0,
                        green: 2,
                        blue: 0,
                    },
                ],
            }
        );
        println!("{:#?}", game);
    }
}

fn part1() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d02/input")?;
    let sum: u32 = content
        .lines()
        .map(|line| {
            let (_, game) = all_consuming(parse_line)(line).unwrap();
            game
        })
        .filter(|Game { cube_sets, .. }| {
            cube_sets
                .iter()
                .all(|CubeSet { red, green, blue }| *red <= 12 && *green <= 13 && *blue <= 14)
        })
        .map(|Game { gameid, .. }| gameid)
        .sum();
    println!("{}", sum);
    Ok(())
}

fn part2() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d02/input")?;
    let sum: u32 = content
        .lines()
        .map(|line| {
            let (_, game) = all_consuming(parse_line)(line).unwrap();
            let (mut minred, mut minblue, mut mingreen) = (0, 0, 0);
            for CubeSet { red, green, blue } in game.cube_sets {
                minred = u32::max(minred, red);
                mingreen = u32::max(mingreen, green);
                minblue = u32::max(minblue, blue);
            }
            minred * minblue * mingreen
        })
        .sum();
    println!("{}", sum);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // part1()
    part2()
}
