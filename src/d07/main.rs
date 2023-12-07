use std::error::Error;
use std::fs;

const HAND_SIZE: usize = 5;
const JOKER: i8 = -1;

#[derive(Debug)]
struct Hand([i8; HAND_SIZE]);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum HandType {
    FiveOfAKind = 7,
    FourOfAKind = 6,
    FullHouse = 5,
    ThreeOfAKind = 4,
    TwoPair = 3,
    OnePair = 2,
    HighCard = 1,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_hand_ordering() {
        let mut hand_types = vec![
            HandType::FullHouse,
            HandType::FiveOfAKind,
            HandType::FourOfAKind,
            HandType::ThreeOfAKind,
            HandType::HighCard,
            HandType::OnePair,
            HandType::TwoPair,
        ];
        hand_types.sort();
        assert_eq!(
            hand_types,
            vec![
                HandType::HighCard,
                HandType::OnePair,
                HandType::TwoPair,
                HandType::ThreeOfAKind,
                HandType::FullHouse,
                HandType::FourOfAKind,
                HandType::FiveOfAKind,
            ]
        )
    }
}

impl Hand {
    fn parse(s: &str) -> Result<Self, String> {
        let mut arr: [i8; HAND_SIZE] = Default::default();
        let mut iter = s.chars();
        for i in 0..5 {
            arr[i] = match iter.next().ok_or("oh no")? {
                '2' => Ok(2),
                '3' => Ok(3),
                '4' => Ok(4),
                '5' => Ok(5),
                '6' => Ok(6),
                '7' => Ok(7),
                '8' => Ok(8),
                '9' => Ok(9),
                'T' => Ok(10),
                'J' => Ok(JOKER),
                'Q' => Ok(12),
                'K' => Ok(13),
                'A' => Ok(14),
                ch => Err(format!("Unexpected char {}", ch)),
            }?;
        }
        Ok(Hand(arr))
    }
    fn hand_type(&self) -> HandType {
        // self.hand_type_no_joker()
        let has_jokers = self.0.iter().any(|&x| x == JOKER);
        if has_jokers {
            let best_type = (2..=14)
                .map(|x| {
                    let mut cloned_arr = self.0.clone();
                    for v in cloned_arr.iter_mut() {
                        if *v == JOKER {
                            *v = x
                        }
                    }
                    Hand(cloned_arr).hand_type_no_joker()
                })
                .max()
                .unwrap();
            best_type
        } else {
            self.hand_type_no_joker()
        }
    }

    fn hand_type_no_joker(&self) -> HandType {
        let mut sorted_arr = self.0.clone();
        sorted_arr.sort();
        let sorted_arr = sorted_arr;

        if sorted_arr[0] == sorted_arr[4] {
            return HandType::FiveOfAKind;
        }

        let is_four_of_a_kind = sorted_arr[0] == sorted_arr[3] || sorted_arr[1] == sorted_arr[4];
        if is_four_of_a_kind {
            // if arr[0] == arr[3] in the SORTED arr that implies a contiguous block of 4 numbers.
            // Likewise for arr[1] == arr[4]
            return HandType::FourOfAKind;
        }

        let uniq_numbers = {
            let mut uniq_numbers = 1; // Start with 1 since the first element is always unique
            for i in 1..sorted_arr.len() {
                if sorted_arr[i] != sorted_arr[i - 1] {
                    uniq_numbers += 1;
                }
            }
            uniq_numbers
        };

        if uniq_numbers == 2 {
            // Two unique numbers AND not four of a kind implies full house
            return HandType::FullHouse;
        }

        let is_three_of_a_kind = (sorted_arr[0] == sorted_arr[2])
            || (sorted_arr[1] == sorted_arr[3])
            || (sorted_arr[2] == sorted_arr[4]);
        if is_three_of_a_kind {
            // Found a contiguous block of 3 numbers AND not four of a kind or five of a kind AND
            // not full house
            return HandType::ThreeOfAKind;
        }

        if uniq_numbers == 3 {
            return HandType::TwoPair;
        }

        if uniq_numbers == 4 {
            return HandType::OnePair;
        }

        return HandType::HighCard;
    }
}

impl PartialEq for Hand {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Hand {}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.hand_type(), self.0).cmp(&(other.hand_type(), other.0))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d07/input")?;

    let mut hands_and_bids = content
        .lines()
        .map(|line| -> Result<_, Box<dyn Error>> {
            let mut iter = line.split(' ');
            let hand = Hand::parse(iter.next().ok_or("oh no".to_owned())?)?;
            let bid = iter.next().ok_or("oh no".to_owned())?.parse::<usize>()?;
            Ok((hand, bid))
        })
        .collect::<Result<Vec<_>, _>>()?;
    // TODO: why doesn't this work?
    // hands_and_bids.sort_by_key(|(_, hand, _)| hand);
    hands_and_bids.sort();
    let s: usize = hands_and_bids
        .iter()
        .enumerate()
        .map(|(i, (_, bid))| (i + 1) * bid)
        .sum();
    println!("{}", s);
    Ok(())
}
