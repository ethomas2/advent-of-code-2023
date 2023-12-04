use nom::IResult;
use nom::{
    bytes::complete::tag,
    character::complete::space0,
    combinator::all_consuming,
    multi::many1,
    sequence::{delimited, terminated, tuple},
};
use std::collections::HashSet;
use std::error::Error;
use std::fs;

#[derive(Debug)]
struct Card {
    winning_numbers: HashSet<i64>,
    my_numbers: HashSet<i64>,
}

fn parse_line(input: &str) -> IResult<&str, Card> {
    let (input, (_, _, winning_numbers, _, my_numbers)) = tuple((
        terminated(tag("Card"), space0),
        delimited(space0, nom::character::complete::i64, tag(":")),
        many1(delimited(space0, nom::character::complete::i64, space0)),
        delimited(space0, tag("|"), space0),
        many1(delimited(space0, nom::character::complete::i64, space0)),
    ))(input)?;

    let winning_numbers: HashSet<i64> = winning_numbers.into_iter().collect();
    let my_numbers: HashSet<i64> = my_numbers.into_iter().collect();
    let card = Card {
        winning_numbers,
        my_numbers,
    };
    Ok((input, card))
}

#[allow(dead_code)]
fn part1() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d04/input")?;
    let cards: Vec<Card> = content
        .lines()
        .map(|line| all_consuming(parse_line)(line).unwrap().1)
        .collect();

    let value: usize = cards
        .iter()
        .map(|card| {
            let Card {
                winning_numbers,
                my_numbers,
                ..
            } = card;
            let intersection = winning_numbers.intersection(&my_numbers);
            let n = intersection.count();
            if n == 0 {
                0
            } else {
                2usize.pow((n - 1).try_into().unwrap())
            }
        })
        .sum();

    println!("{}", value);

    Ok(())
}

fn part2() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d04/input")?;
    let cards: Vec<Card> = content
        .lines()
        .map(|line| all_consuming(parse_line)(line).unwrap().1)
        .collect();

    // tuple of (<number of copies, <card>)
    let mut state: Vec<(usize, Card)> = cards.into_iter().map(|card| (1, card)).collect();

    // process the cards
    let mut idx = 0;
    while idx < state.len() {
        let (before, after) = state.split_at_mut(idx + 1);
        let (copies, card) = before.last().unwrap(); // card at current idx
        let wins = card.winning_numbers.intersection(&card.my_numbers).count();
        for next_card_idx in 0..wins {
            after[next_card_idx].0 += copies;
        }
        idx += 1;
    }

    // how many copies did you end up with
    let total_copies: usize = state.iter().map(|(ncopies, _)| ncopies).sum();
    println!("{}", total_copies);
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // part1()
    part2()
}
