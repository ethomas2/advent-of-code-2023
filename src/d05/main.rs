use std::collections::HashMap;
use std::error::Error;
use std::fs;

use nom::IResult;
use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, newline, space0, space1},
    combinator::{all_consuming, opt},
    multi::{many0, many1},
    sequence::{terminated, tuple},
};

#[derive(Debug, PartialEq, Eq)]
struct MappedRange {
    src_start: u64,
    dst_start: u64,
    length: u64,
}
impl MappedRange {
    fn src_contains(&self, val: u64) -> bool {
        // TODO: what's the rust syntax to inline this?
        // TODO: is this less performant than src_start <= val && val < end?
        let end = self.src_start + self.length;
        (self.src_start..end).contains(&val)
    }
}
#[derive(Debug, PartialEq, Eq)]
struct Map<'a> {
    src_type: &'a str,
    dst_type: &'a str,

    ranges: Vec<MappedRange>,
}

impl<'a> Map<'a> {
    fn transform(&self, val: u64) -> u64 {
        for range in self.ranges.iter() {
            if range.src_contains(val) {
                return range.dst_start + (val - range.src_start);
            }
        }
        return val;
    }
}

type AllMaps<'a> = HashMap<String, Map<'a>>;
/// Parse input. Input looks like the following
///
///     seeds: 79 14 55 13
///
///     seed-to-soil map:
///     50 98 2
///     52 50 48
///
///     soil-to-fertilizer map:
///     0 15 37
///     37 52 2
///     39 0 15
fn parse<'a>(input: &'a str) -> IResult<&str, (Vec<u64>, AllMaps)> {
    // seeds: 79 14 55 13
    let (input, (_, _, _, seeds)) = tuple((
        tag("seeds"),
        tag(":"),
        space0,
        many1(terminated(nom::character::complete::u64, space0)),
    ))(input)?;

    // seed-to-soil map:
    // 50 98 2
    // 52 50 48
    let (input, maps) = many1(|input: &'a str| -> IResult<&str, Map> {
        let (input, _) = many0(newline)(input)?;

        // seed-to-soil-map:
        let (input, (src_type, _, dst_type, _, _, _)) =
            tuple((alpha1, tag("-to-"), alpha1, space1, tag("map:"), newline))(input)?;

        // list of numbers
        let (input, ranges) = many1(tuple((
            terminated(nom::character::complete::u64, space0),
            terminated(nom::character::complete::u64, space0),
            terminated(nom::character::complete::u64, opt(newline)),
        )))(input)?;

        let ranges = ranges
            .iter()
            .map(|&(dst_start, src_start, length)| MappedRange {
                src_start,
                dst_start,
                length,
            })
            .collect();

        let map = Map {
            src_type,
            dst_type,
            ranges,
        };
        Ok((input, map))
    })(input)?;

    // TODO: do the keys have to be String? Can they be &str?
    let maps: HashMap<String, Map> = maps
        .into_iter()
        .map(|map| (map.src_type.to_owned(), map))
        .collect();
    Ok((input, (seeds, maps)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        let input = "\
seeds: 79 14 55 13

seed-to-soil map:
50 98 2
52 50 48

soil-to-fertilizer map:
0 15 37
37 52 2
39 0 15";
        let x = parse(input);
        match x {
            Ok((remainder, (seeds, maps))) => {
                assert_eq!(remainder.len(), 0, "Failed to consume entire input");
                assert_eq!(seeds, vec![79, 14, 55, 13]);
                assert!(maps.len() == 2);
                assert_eq!(
                    *maps.get("seed").unwrap(), // TODO: why can I derefernce here?
                    Map {
                        src_type: "seed",
                        dst_type: "soil",
                        ranges: vec![
                            MappedRange {
                                src_start: 98,
                                dst_start: 50,
                                length: 2
                            },
                            MappedRange {
                                src_start: 50,
                                dst_start: 52,
                                length: 48
                            }
                        ],
                    }
                );

                assert_eq!(
                    *maps.get("soil").unwrap(), // TODO: why can I derefernce here?
                    Map {
                        src_type: "soil",
                        dst_type: "fertilizer",
                        ranges: vec![
                            MappedRange {
                                src_start: 15,
                                dst_start: 0,
                                length: 37
                            },
                            MappedRange {
                                src_start: 52,
                                dst_start: 37,
                                length: 2
                            },
                            MappedRange {
                                src_start: 0,
                                dst_start: 39,
                                length: 15
                            }
                        ],
                    }
                );
            }
            Err(x) => {
                assert!(false, "{:?}", x);
            }
        }
    }
}

fn transform(mut seed: u64, maps: &AllMaps) -> Result<u64, String> {
    let mut map = maps
        .get("seed")
        .ok_or("Could not find seed map".to_owned())?;
    loop {
        seed = map.transform(seed);
        if map.dst_type == "location" {
            break;
        }
        map = maps
            .get(map.dst_type)
            .ok_or(format!("Could not find {} map", map.dst_type))?
    }
    Ok(seed)
}

fn main() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d05/input")?;
    let (_, (seeds, maps)) = all_consuming(parse)(content.as_ref()).expect("Could not parse input");

    // TODO: use process results instead
    let transformed = seeds
        .iter()
        .map(|seed| transform(*seed, &maps))
        .collect::<Result<Vec<_>, _>>()?;

    let min = transformed.iter().min().expect("0 transformed values");
    println!("min {}", min);
    Ok(())
}
