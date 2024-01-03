use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, newline, space0, space1},
    combinator::{all_consuming, map},
    multi::{many0, separated_list1},
    sequence::{terminated, tuple},
    IResult,
};
use std::collections::HashMap;

// Parse::ModuleType
//      Broadcaster
//      FlipFlop
//      Conjunction
//
// Parse::ModuleMap // map from module ident -> (type for module, list of connections)
//      HashMap<&str, (ParseType, Vec<&str>)>

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ModuleType {
    Broadcaster,
    FlipFlop,
    Conjunction,
}

pub type ModuleMap<'a> = HashMap<&'a str, (ModuleType, Vec<&'a str>)>;

pub fn parse_line(input: &str) -> IResult<&str, (&str, ModuleType, Vec<&str>)> {
    // Example input
    //
    // broadcaster -> a, b, c
    // %a -> b
    // %b -> c
    // %c -> inv
    // &inv -> a

    let (input, modtype) =
        map(
            alt((tag("broadcaster"), tag("%"), tag("&"))),
            |modtype| match modtype {
                "broadcaster" => ModuleType::Broadcaster,
                "%" => ModuleType::FlipFlop,
                "&" => ModuleType::Conjunction,
                _ => panic!("logic error"),
            },
        )(input)?;
    let (input, module_name) = match modtype {
        ModuleType::Broadcaster => (input, "broadcaster"),
        _ => alpha1(input)?,
    };
    let (input, _) = tuple((space1, tag("->"), space1))(input)?;
    let (input, connections) = separated_list1(tuple((tag(","), space0)), alpha1)(input)?;

    Ok((input, (module_name, modtype, connections)))
}

pub fn parse(input: &str) -> IResult<&str, ModuleMap> {
    let (input, lines) = all_consuming(terminated(
        separated_list1(newline, parse_line),
        many0(newline),
    ))(input)?;

    let mmap: ModuleMap = lines
        .into_iter()
        .map(|(src, modtype, dst)| (src, (modtype, dst)))
        .collect();

    Ok((input, mmap))
}
