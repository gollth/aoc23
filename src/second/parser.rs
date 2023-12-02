use crate::second::{Color, Draw, Game, Round};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{space1, u32},
    combinator::{map, value},
    multi::separated_list0,
    sequence::{preceded, terminated},
    IResult, Parser as NomParser,
};

pub(crate) fn parse_game(s: &str) -> IResult<&str, Game> {
    let (s, id) = preceded(tag("Game "), terminated(u32, tag(": ")))(s)?;
    let (s, rounds) = separated_list0(tag("; "), parse_round)(s)?;
    Ok((s, Game { id, rounds }))
}

fn parse_round(s: &str) -> IResult<&str, Round> {
    map(separated_list0(tag(", "), parse_draw), |xs| {
        Round(xs.into_iter().collect())
    })(s)
}

fn parse_draw(s: &str) -> IResult<&str, Draw> {
    map(
        u32.and(preceded(
            space1,
            alt((
                value(Color::Blue, tag("blue")),
                value(Color::Red, tag("red")),
                value(Color::Green, tag("green")),
            )),
        )),
        |(n, color)| (color, n),
    )(s)
}
