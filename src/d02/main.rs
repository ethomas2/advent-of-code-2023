use std::error::Error;
use std::fs;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{space0, space1, u32 as nomu32},
    combinator::{all_consuming, opt},
    multi::many1,
    IResult,
};

#[derive(Debug)]
struct CubeSet {
    red: u32,
    green: u32,
    blue: u32,
}

#[derive(Debug)]
struct Game {
    gameid: u32,
    cube_sets: Vec<CubeSet>,
}

fn parse_cube_set(input: &str) -> IResult<&str, CubeSet> {
    let (input, cube_tuples) = many1(|input| -> IResult<&str, (&str, u32)> {
        let (input, n) = nomu32(input)?;
        let (input, _) = space1(input)?;
        let (input, color_name) = alt((tag("red"), tag("blue"), tag("green")))(input)?;
        let (input, _) = opt(tag(","))(input)?;
        let (input, _) = space0(input)?;
        Ok((input, (color_name, n)))
    })(input)?;

    let mut cubeset = CubeSet {
        red: 0,
        green: 0,
        blue: 0,
    };
    for (cube_name, n) in cube_tuples {
        match cube_name {
            "red" => {
                cubeset.red = n;
            }
            "green" => {
                cubeset.green = n;
            }
            "blue" => {
                cubeset.blue = n;
            }
            _ => unreachable!(),
        }
    }

    Ok((input, cubeset))
}

fn parse_line(input: &str) -> IResult<&str, Game> {
    let (input, _) = tag("Game")(input)?;
    let (input, _) = space1(input)?;
    let (input, gameid) = nomu32(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, _) = space1(input)?;
    let (input, cube_sets) = many1(|input| -> IResult<&str, _> {
        let (input, cubeset) = parse_cube_set(input)?;
        let (input, _) = opt(tag(";"))(input)?;
        let (input, _) = space0(input)?;
        Ok((input, cubeset))
    })(input)?;
    Ok((input, Game { gameid, cube_sets }))
}

fn main() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d02/input")?;
    let sum: u32 = content
        .lines()
        .map(|line| {
            // TODO: all_consuming.finish()?
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
