use itertools::Itertools;
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

// using my own custom range class instead of rust's built in range class so i can implement
// overlaps(). [start, end)
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
struct Range {
    start: u64,
    end: u64,
}

impl Range {
    fn new(start: u64, end: u64) -> Range {
        assert!(start <= end, "start {} is not <= end {}", start, end);
        Range { start, end }
    }
    fn len(&self) -> u64 {
        self.end - self.start
    }
    fn contains(&self, val: u64) -> bool {
        self.start < val && val < self.end
    }

    fn overlaps(&self, other: &Range) -> bool {
        !((self.end <= other.start) || (other.end <= self.start))
    }

    fn debug_assert_ranges_disjoint(ranges: &Vec<Range>) {
        let mut ranges = ranges.clone();
        ranges.sort();
        for chunk in ranges.as_slice().chunks_exact(2) {
            let (range1, range2) = (&chunk[0], &chunk[1]);
            debug_assert!(
                range1.end <= range2.start,
                "seed ranges are not disjoint {:?} {:?}",
                range1,
                range2
            );
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Map<'a> {
    src_type: &'a str,
    dst_type: &'a str,

    src_ranges: Vec<Range>,
    dst_ranges: Vec<Range>,
}

impl<'a> Map<'a> {
    fn new(
        src_type: &'a str,
        dst_type: &'a str,
        src_ranges: Vec<Range>,
        dst_ranges: Vec<Range>,
    ) -> Map<'a> {
        // If ranges are not disjoin that means a single element is mapped to two values
        Range::debug_assert_ranges_disjoint(&src_ranges);
        Range::debug_assert_ranges_disjoint(&dst_ranges);
        debug_assert!(src_ranges.len() == dst_ranges.len());
        debug_assert!(src_ranges
            .iter()
            .zip(dst_ranges.iter())
            .all(|(src_range, dst_range)| src_range.len() == dst_range.len()));
        Map {
            src_type,
            dst_type,
            src_ranges,
            dst_ranges,
        }
    }

    fn transform(&self, val: u64) -> u64 {
        for (src_range, dst_range) in self.src_ranges.iter().zip(self.dst_ranges.iter()) {
            if src_range.contains(val) {
                return dst_range.start + (val - src_range.start);
            }
        }
        return val;
    }
}

type AllMaps<'a> = HashMap<&'a str, Map<'a>>;

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

        let src_ranges: Vec<_> = ranges
            .iter()
            .map(|&(_, src_start, length)| Range::new(src_start, src_start + length))
            .collect();

        let dst_ranges: Vec<_> = ranges
            .iter()
            .map(|&(dst_start, _, length)| Range::new(dst_start, dst_start + length))
            .collect();

        let map = Map::new(src_type, dst_type, src_ranges, dst_ranges);
        Ok((input, map))
    })(input)?;

    let maps: HashMap<&'a str, Map> = maps.into_iter().map(|map| (map.src_type, map)).collect();
    Ok((input, (seeds, maps)))
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

fn transform_range(map: &Map, input_range: &Range) -> Vec<Range> {
    let combined_ranges: Vec<_> = map
        .src_ranges
        .clone()
        .into_iter()
        .zip(map.dst_ranges.clone())
        .collect();
    let mut combined_ranges = combined_ranges.clone();
    combined_ranges.sort_by_key(|(src_range, _)| src_range.start);

    let mut relevant_src_ranges = Vec::new();
    let mut relevant_dst_ranges = Vec::new();
    for (src_range, dst_range) in combined_ranges {
        // filter by relevancy
        if src_range.overlaps(&input_range) {
            relevant_src_ranges.push(src_range);
            relevant_dst_ranges.push(dst_range);
        }
    }

    if relevant_src_ranges.len() == 0 {
        return vec![input_range.clone()];
    }
    // truncate first src_range and dst_range to be in line with input range
    // truncate last src_range and dst_range to be in line with input range
    let left_delta = input_range
        .start
        .saturating_sub(relevant_src_ranges[0].start);
    relevant_src_ranges[0].start += left_delta;
    relevant_dst_ranges[0].start += left_delta;

    let right_delta = relevant_src_ranges
        .last()
        .unwrap()
        .end
        .saturating_sub(input_range.end);
    (*relevant_src_ranges.last_mut().unwrap()).end -= right_delta;
    (*relevant_dst_ranges.last_mut().unwrap()).end -= right_delta;

    let output_ranges: Vec<Range> = {
        let mut output_ranges: Vec<Range> = Vec::new();
        // output all the dst ranges and
        for dst_range in relevant_dst_ranges {
            output_ranges.push(dst_range);
        }

        // output all the "identity ranges" between src ranges
        output_ranges.push(Range::new(input_range.start, relevant_src_ranges[0].start));
        relevant_src_ranges.as_slice().windows(2).for_each(|slice| {
            output_ranges.push(Range::new(slice[0].end, slice[1].start));
        });
        output_ranges.push(Range::new(
            relevant_src_ranges.last().unwrap().end,
            input_range.end,
        ));

        // filter out any empty ranges
        output_ranges = output_ranges
            .into_iter()
            .filter(|r| r.start != r.end)
            .collect();

        // sort
        output_ranges.sort();

        output_ranges
    };

    output_ranges
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_transform_non_overlapping_ranges() {
        // input range 0 - 100
        // map
        //   10 - 20 : 55 - 65
        //   50 - 60 : 80 - 90
        //   60 - 70 : 10 - 20
        //
        //   output
        //     identity ranges
        //          0 - 10
        //          20 - 50
        //          70 - 100
        //
        //      mapped ranges
        //          55 - 65
        //          80 - 90
        //          10 - 20

        let input_range = Range { start: 0, end: 100 };
        let map = Map::new(
            "foo",
            "bar",
            vec![Range::new(10, 20), Range::new(50, 60), Range::new(60, 70)],
            vec![Range::new(55, 65), Range::new(80, 90), Range::new(10, 20)],
        );

        let output_ranges = {
            let mut output_ranges = transform_range(&map, &input_range);
            output_ranges.sort();
            output_ranges
        };

        let expected_output_ranges = {
            let mut v = vec![
                Range::new(0, 10),
                Range::new(20, 50),
                Range::new(70, 100),
                Range::new(55, 65),
                Range::new(80, 90),
                Range::new(10, 20),
            ];
            v.sort();
            v
        };

        assert_eq!(expected_output_ranges, output_ranges);
    }

    #[test]
    fn test_transform_overlapping_ranges() {
        // input range 50 - 100
        //
        // map
        //   45 - 55  : 200 - 210
        //   55 - 60  : 0 - 5
        //   95 - 105 : 40 - 50
        //
        //   output
        //     identity ranges
        //          60 - 95
        //
        //      mapped ranges
        //          205 - 210
        //          0   - 5
        //          40 - 45

        let input_range = Range {
            start: 50,
            end: 100,
        };
        let map = Map::new(
            "foo",
            "bar",
            vec![Range::new(45, 55), Range::new(55, 60), Range::new(95, 105)],
            vec![Range::new(200, 210), Range::new(0, 5), Range::new(40, 50)],
        );

        let output_ranges = {
            let mut output_ranges = transform_range(&map, &input_range);
            output_ranges.sort();
            output_ranges
        };

        let expected_output_ranges = {
            let mut v = vec![
                Range::new(60, 95),
                Range::new(205, 210),
                Range::new(0, 5),
                Range::new(40, 45),
            ];
            v.sort();
            v
        };

        assert_eq!(expected_output_ranges, output_ranges);
    }

    #[test]
    fn test_answers() {
        assert_eq!(part1().unwrap(), 322500873);
        assert_eq!(part2().unwrap(), 108956227);
    }
}

fn part1() -> Result<u64, Box<dyn Error>> {
    let content = fs::read_to_string("src/d05/input")?;
    let (_, (seeds, maps)) = all_consuming(parse)(content.as_ref()).expect("Could not parse input");

    let min = seeds
        .iter()
        .map(|seed| transform(*seed, &maps))
        .process_results(|transformed| transformed.min().expect("0 transformed values"))?;

    Ok(min)
}

fn part2() -> Result<u64, Box<dyn Error>> {
    // Get ranges from input
    // let Transform = Map
    // let SeedRanges = [[Range]]
    // define f: Transform -> Range -> [Range]
    // let f1, f2, f3 = f(Transform1), f(Transform2), f(Transform3)
    // resulting ranges =  (f1 * f2 * f3)(SeedRanges)
    let content = fs::read_to_string("src/d05/input")?;
    let (_, (seed_input, maps)) =
        all_consuming(parse)(content.as_ref()).expect("Could not parse input");
    assert!(seed_input.len() % 2 == 0);

    // TODO: why can't you call chunks_exact directly on Vec?
    let mut ranges: Vec<_> = seed_input
        .as_slice()
        .chunks_exact(2)
        .map(|chunk| {
            let start = chunk[0];
            let length = chunk[1];
            Range::new(start, start + length)
        })
        .collect();

    // assert that ranges are disjoint
    ranges.sort();
    Range::debug_assert_ranges_disjoint(&ranges);

    // map ranges through all the maps
    let mut map = maps
        .get("seed")
        .ok_or("Could not find seed map".to_owned())?;
    loop {
        ranges = ranges
            .iter()
            .flat_map(|range| transform_range(map, range))
            .collect();
        if map.dst_type == "location" {
            break;
        }
        map = maps
            .get(map.dst_type)
            .ok_or(format!("Could not find {} map", map.dst_type))?
    }
    let min_value = ranges.iter().map(|range| range.start).min().unwrap();

    Ok(min_value)
}
fn main() -> Result<(), Box<dyn Error>> {
    println!("Part 1 answer: {}", part1().unwrap());
    println!("Part 2 answer: {}", part2().unwrap());
    Ok(())
}
