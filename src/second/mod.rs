mod parser;

use crate::second::parser::parse_game;
use anyhow::anyhow;
use nom::Finish;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Color {
    Blue,
    Red,
    Green,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Game {
    id: u32,
    rounds: Vec<Round>,
}

impl Game {
    pub fn possible(&self, bag: &HashMap<Color, u32>) -> bool {
        self.rounds.iter().all(|round| {
            round
                .0
                .iter()
                .all(|(color, n)| n <= bag.get(color).unwrap_or(&0))
        })
    }
    pub fn id(&self) -> u32 {
        self.id
    }
}
impl FromStr for Game {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(parse_game(s).finish().map_err(|e| anyhow!("{e}"))?.1)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Round(HashMap<Color, u32>);

pub type Draw = (Color, u32);

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("Game 1: ", Game { id: 1, rounds: vec![Round(HashMap::new())] })]
    #[case("Game 2: 3 blue", Game { id: 2, rounds: vec![Round([(Color::Blue, 3)].into_iter().collect())] })]
    #[case("Game 3: 5 blue, 1 green", Game { id: 3, rounds: vec![Round([(Color::Blue, 5), (Color::Green, 1)].into_iter().collect())] })]
    #[case("Game 4: 8 blue, 3 green, 2 red", Game { id: 4, rounds: vec![Round([(Color::Blue, 8), (Color::Green, 3), (Color::Red, 2)].into_iter().collect())] })]
    #[case("Game 5: 8 blue; 3 green; 2 red", Game { id: 5, rounds: vec![
        Round([(Color::Blue, 8)].into_iter().collect()), 
        Round([(Color::Green, 3)].into_iter().collect()), 
        Round([(Color::Red, 2)].into_iter().collect()), 
    ]})]
    fn game_fromstr(#[case] s: &str, #[case] expected: Game) {
        assert_eq!(expected, Game::from_str(s).unwrap());
    }
}
