use aoc23::Part;

use anyhow::{anyhow, Result};
use clap::Parser;
use itertools::Itertools;
use std::{cmp::Ordering, collections::HashMap, fmt::Display, str::FromStr};

/// Day 7: Camel Cards
#[derive(Debug, Parser)]
struct Options {
    /// Path to the file with the input data
    #[clap(short, long, default_value = "sample/seventh.txt")]
    input: String,

    /// Which part of the day to solve
    part: Part,
}

fn main() -> Result<()> {
    let args = Options::parse();
    let input = std::fs::read_to_string(&args.input)?;
    let mut game = Game::from_str(&input)?;
    let solution = match args.part {
        Part::One => game
            .ranking()
            .map(|(_, bid)| bid)
            .zip(1..)
            .map(|(bid, rank)| bid * rank)
            .sum::<u32>(),
        Part::Two => unimplemented!(),
    };
    println!("Solution part {part:?}: {solution}", part = args.part);
    Ok(())
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum Face {
    Ace,
    King,
    Queen,
    Jack,
    Number(u8),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
enum Rank {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Hand {
    cards: [Face; 5],
    rank: Rank,
}

type Bid = u32;

#[derive(Debug, PartialEq, Eq)]
struct Game {
    rounds: Vec<(Hand, Bid)>,
}

impl FromStr for Game {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let rounds = s
            .lines()
            .map(|line| {
                let (hand, bid) = line
                    .split_whitespace()
                    .next_tuple()
                    .ok_or(anyhow!("Expected two elements defining a game"))?;
                Ok((Hand::from_str(hand)?, bid.parse::<Bid>()?))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Game { rounds })
    }
}

impl Game {
    fn ranking(&mut self) -> impl Iterator<Item = &(Hand, Bid)> + '_ {
        self.rounds.sort_by_key(|r| r.0.clone());
        self.rounds.iter()
    }
}

impl From<[Face; 5]> for Rank {
    fn from(cards: [Face; 5]) -> Self {
        let groups = cards.iter().fold(HashMap::new(), |mut cache, card| {
            *cache.entry(card).or_insert(0) += 1;
            cache
        });
        match groups.len() {
            5 => Self::HighCard,
            4 => Self::OnePair,
            3 if groups.values().any(|n| *n == 3) => Self::ThreeOfAKind,
            3 => Self::TwoPair,
            2 if groups.values().any(|n| *n == 4) => Self::FourOfAKind,
            2 => Self::FullHouse,
            1 => Self::FiveOfAKind,
            _ => unreachable!(),
        }
    }
}

impl FromStr for Hand {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > 5 {
            return Err(anyhow!("Hands consists only of 5 cards"));
        }
        let cards = s
            .chars()
            .map(Face::try_from)
            .collect::<Result<Vec<Face>>>()?
            .as_slice()
            .try_into()?;
        Ok(Self {
            cards,
            rank: cards.into(),
        })
    }
}

impl Display for Hand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}{}",
            self.cards[0], self.cards[1], self.cards[2], self.cards[3], self.cards[4]
        )
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.rank.partial_cmp(&other.rank)? {
            Ordering::Equal => self
                .cards
                .iter()
                .zip(other.cards.iter())
                .find(|(a, b)| a.cmp(b) != Ordering::Equal)
                .map(|(a, b)| a.partial_cmp(b))
                .unwrap_or(Some(Ordering::Equal)),
            o => Some(o),
        }
    }
}
impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl TryFrom<char> for Face {
    type Error = anyhow::Error;
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'A' => Ok(Self::Ace),
            'K' => Ok(Self::King),
            'Q' => Ok(Self::Queen),
            'J' => Ok(Self::Jack),
            'T' => Ok(Self::Number(10)),
            n => Ok(Self::Number(
                n.to_digit(10).ok_or(anyhow!("Expected digit found {n}"))? as u8,
            )),
        }
    }
}
impl Display for Face {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Face::King => write!(f, "K"),
            Face::Queen => write!(f, "Q"),
            Face::Jack => write!(f, "J"),
            Face::Number(10) => write!(f, "T"),
            Face::Number(n) => write!(f, "{}", n),
            Face::Ace => write!(f, "A"),
        }
    }
}
impl Ord for Face {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for Face {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(match (*self, *other) {
            (Self::Ace, Self::Ace) => Ordering::Equal,
            (Self::Ace, _) => Ordering::Greater,
            (Self::King, Self::Ace) => Ordering::Less,
            (Self::King, Self::King) => Ordering::Equal,
            (Self::King, _) => Ordering::Greater,
            (Self::Queen, Self::Ace) => Ordering::Less,
            (Self::Queen, Self::King) => Ordering::Less,
            (Self::Queen, Self::Queen) => Ordering::Equal,
            (Self::Queen, _) => Ordering::Greater,
            (Self::Jack, Self::King) => Ordering::Less,
            (Self::Jack, Self::Ace) => Ordering::Less,
            (Self::Jack, Self::Queen) => Ordering::Less,
            (Self::Jack, Self::Jack) => Ordering::Equal,
            (Self::Jack, _) => Ordering::Greater,
            (Self::Number(a), Self::Number(b)) => a.cmp(&b),
            (Self::Number(_), _) => Ordering::Less,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use itertools::izip;
    use rstest::rstest;

    #[rstest]
    #[case('A', Ordering::Equal, 'A')]
    #[case('A', Ordering::Greater, 'K')]
    #[case('A', Ordering::Greater, 'Q')]
    #[case('A', Ordering::Greater, 'J')]
    #[case('A', Ordering::Greater, 'T')]
    #[case('A', Ordering::Greater, '9')]
    #[case('A', Ordering::Greater, '2')]
    #[case('K', Ordering::Less, 'A')]
    #[case('K', Ordering::Equal, 'K')]
    #[case('K', Ordering::Greater, 'Q')]
    #[case('K', Ordering::Greater, 'J')]
    #[case('K', Ordering::Greater, 'T')]
    #[case('K', Ordering::Greater, '9')]
    #[case('K', Ordering::Greater, '2')]
    #[case('Q', Ordering::Less, 'A')]
    #[case('Q', Ordering::Less, 'K')]
    #[case('Q', Ordering::Equal, 'Q')]
    #[case('Q', Ordering::Greater, 'J')]
    #[case('Q', Ordering::Greater, 'T')]
    #[case('Q', Ordering::Greater, '9')]
    #[case('Q', Ordering::Greater, '2')]
    #[case('J', Ordering::Less, 'A')]
    #[case('J', Ordering::Less, 'K')]
    #[case('J', Ordering::Less, 'Q')]
    #[case('J', Ordering::Equal, 'J')]
    #[case('J', Ordering::Greater, 'T')]
    #[case('J', Ordering::Greater, '9')]
    #[case('J', Ordering::Greater, '2')]
    #[case('T', Ordering::Less, 'K')]
    #[case('T', Ordering::Less, 'A')]
    #[case('T', Ordering::Less, 'Q')]
    #[case('T', Ordering::Less, 'J')]
    #[case('T', Ordering::Equal, 'T')]
    #[case('T', Ordering::Greater, '9')]
    #[case('T', Ordering::Greater, '2')]
    #[case('7', Ordering::Less, 'K')]
    #[case('7', Ordering::Less, 'A')]
    #[case('7', Ordering::Less, 'Q')]
    #[case('7', Ordering::Less, 'J')]
    #[case('7', Ordering::Less, 'T')]
    #[case('7', Ordering::Less, '9')]
    #[case('7', Ordering::Equal, '7')]
    #[case('7', Ordering::Greater, '2')]
    fn face_ord(#[case] a: char, #[case] expected: Ordering, #[case] b: char) {
        assert_eq!(
            expected,
            Face::try_from(a).unwrap().cmp(&Face::try_from(b).unwrap()),
            "{a} {expected:?} {b}"
        );
    }

    #[rstest]
    // FiveOfAKind
    #[case(Rank::FiveOfAKind, Ordering::Equal, Rank::FiveOfAKind)]
    #[case(Rank::FiveOfAKind, Ordering::Greater, Rank::FourOfAKind)]
    #[case(Rank::FiveOfAKind, Ordering::Greater, Rank::FullHouse)]
    #[case(Rank::FiveOfAKind, Ordering::Greater, Rank::ThreeOfAKind)]
    #[case(Rank::FiveOfAKind, Ordering::Greater, Rank::TwoPair)]
    #[case(Rank::FiveOfAKind, Ordering::Greater, Rank::OnePair)]
    #[case(Rank::FiveOfAKind, Ordering::Greater, Rank::HighCard)]
    // FourOfAKind
    #[case(Rank::FourOfAKind, Ordering::Less, Rank::FiveOfAKind)]
    #[case(Rank::FourOfAKind, Ordering::Equal, Rank::FourOfAKind)]
    #[case(Rank::FourOfAKind, Ordering::Greater, Rank::FullHouse)]
    #[case(Rank::FourOfAKind, Ordering::Greater, Rank::ThreeOfAKind)]
    #[case(Rank::FourOfAKind, Ordering::Greater, Rank::TwoPair)]
    #[case(Rank::FourOfAKind, Ordering::Greater, Rank::OnePair)]
    #[case(Rank::FourOfAKind, Ordering::Greater, Rank::HighCard)]
    // FullHouse
    #[case(Rank::FullHouse, Ordering::Less, Rank::FiveOfAKind)]
    #[case(Rank::FullHouse, Ordering::Less, Rank::FourOfAKind)]
    #[case(Rank::FullHouse, Ordering::Equal, Rank::FullHouse)]
    #[case(Rank::FullHouse, Ordering::Greater, Rank::ThreeOfAKind)]
    #[case(Rank::FullHouse, Ordering::Greater, Rank::TwoPair)]
    #[case(Rank::FullHouse, Ordering::Greater, Rank::OnePair)]
    #[case(Rank::FullHouse, Ordering::Greater, Rank::HighCard)]
    // ThreeOfAKind
    #[case(Rank::ThreeOfAKind, Ordering::Less, Rank::FiveOfAKind)]
    #[case(Rank::ThreeOfAKind, Ordering::Less, Rank::FourOfAKind)]
    #[case(Rank::ThreeOfAKind, Ordering::Less, Rank::FullHouse)]
    #[case(Rank::ThreeOfAKind, Ordering::Equal, Rank::ThreeOfAKind)]
    #[case(Rank::ThreeOfAKind, Ordering::Greater, Rank::TwoPair)]
    #[case(Rank::ThreeOfAKind, Ordering::Greater, Rank::OnePair)]
    #[case(Rank::ThreeOfAKind, Ordering::Greater, Rank::HighCard)]
    // TwoPair
    #[case(Rank::TwoPair, Ordering::Less, Rank::FiveOfAKind)]
    #[case(Rank::TwoPair, Ordering::Less, Rank::FourOfAKind)]
    #[case(Rank::TwoPair, Ordering::Less, Rank::FullHouse)]
    #[case(Rank::TwoPair, Ordering::Less, Rank::ThreeOfAKind)]
    #[case(Rank::TwoPair, Ordering::Equal, Rank::TwoPair)]
    #[case(Rank::TwoPair, Ordering::Greater, Rank::OnePair)]
    #[case(Rank::TwoPair, Ordering::Greater, Rank::HighCard)]
    // OnePair
    #[case(Rank::OnePair, Ordering::Less, Rank::FiveOfAKind)]
    #[case(Rank::OnePair, Ordering::Less, Rank::FourOfAKind)]
    #[case(Rank::OnePair, Ordering::Less, Rank::FullHouse)]
    #[case(Rank::OnePair, Ordering::Less, Rank::ThreeOfAKind)]
    #[case(Rank::OnePair, Ordering::Less, Rank::TwoPair)]
    #[case(Rank::OnePair, Ordering::Equal, Rank::OnePair)]
    #[case(Rank::OnePair, Ordering::Greater, Rank::HighCard)]
    // HighCard
    #[case(Rank::HighCard, Ordering::Less, Rank::FiveOfAKind)]
    #[case(Rank::HighCard, Ordering::Less, Rank::FourOfAKind)]
    #[case(Rank::HighCard, Ordering::Less, Rank::FullHouse)]
    #[case(Rank::HighCard, Ordering::Less, Rank::ThreeOfAKind)]
    #[case(Rank::HighCard, Ordering::Less, Rank::TwoPair)]
    #[case(Rank::HighCard, Ordering::Less, Rank::OnePair)]
    #[case(Rank::HighCard, Ordering::Equal, Rank::HighCard)]
    fn rank_ord(#[case] a: Rank, #[case] expected: Ordering, #[case] b: Rank) {
        assert_eq!(expected, a.cmp(&b), "{a:?} {expected:?} {b:?}");
    }

    #[rstest]
    #[case("23456", Rank::HighCard)]
    #[case("Q3T5J", Rank::HighCard)]
    #[case("Q3TQJ", Rank::OnePair)]
    #[case("Q3TQ3", Rank::TwoPair)]
    #[case("12TTT", Rank::ThreeOfAKind)]
    #[case("K3K2K", Rank::ThreeOfAKind)]
    #[case("T777T", Rank::FullHouse)]
    #[case("AJAAJ", Rank::FullHouse)]
    #[case("QQQQ7", Rank::FourOfAKind)]
    #[case("QQ7QQ", Rank::FourOfAKind)]
    #[case("AAAAA", Rank::FiveOfAKind)]
    #[case("33333", Rank::FiveOfAKind)]
    fn hand_rank(#[case] hand: Hand, #[case] rank: Rank) {
        assert_eq!(rank, hand.rank);
    }

    #[rstest]
    #[case("AAAAA", Ordering::Equal, "AAAAA")]
    #[case("7AAAA", Ordering::Less, "AAAAA")]
    #[case("AAAAA", Ordering::Greater, "7AAAA")]
    #[case("QQKKT", Ordering::Greater, "KKTJQ")]
    #[case("QQKKT", Ordering::Greater, "JJQQA")]
    #[case("QQKKT", Ordering::Less, "QQATA")]
    #[case("QQKKT", Ordering::Less, "QQKAK")]
    #[case("55T22", Ordering::Less, "55A11")]
    #[case("55T22", Ordering::Greater, "5511A")]
    fn hand_ord(#[case] a: Hand, #[case] expected: Ordering, #[case] b: Hand) {
        assert_eq!(expected, a.cmp(&b), "{a:?} {expected:?} {b:?}");
    }

    #[rstest]
    fn sample_a_manual() {
        let input = include_str!("../../sample/seventh.txt");
        let mut game = Game::from_str(input).expect("parsing");
        for (rank, (hand, bid), (expected_hand, expected_bid)) in izip!(
            1..,
            game.ranking(),
            &[
                ("32T3K", 765),
                ("KTJJT", 220),
                ("KK677", 28),
                ("T55J5", 684),
                ("QQQJA", 483)
            ]
        ) {
            assert_eq!(expected_hand, &hand.to_string(), "Rank #{rank}");
            assert_eq!(expected_bid, bid, "Rank #{rank}");
        }
    }

    #[rstest]
    fn sample_a() {
        let input = include_str!("../../sample/seventh.txt");
        let mut game = Game::from_str(input).expect("parsing");
        let solution = game
            .ranking()
            .map(|(_, bid)| bid)
            .zip(1..)
            .map(|(bid, rank)| bid * rank)
            .sum::<u32>();
        assert_eq!(6440, solution);
    }
}
