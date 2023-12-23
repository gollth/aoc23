use nom::{
    character::complete::{alpha1, char, digit1},
    multi::separated_list1,
    sequence::tuple,
    IResult, Parser as NomParser,
};
use nom_supreme::ParserExt;

use super::{Label, Operation};

pub(crate) fn operation(s: &str) -> IResult<&str, Operation> {
    char('-')
        .value(Operation::Remove)
        .or(char('=')
            .precedes(digit1.map_res(str::parse))
            .map(Operation::Insert))
        .parse(s)
}

pub(crate) fn label(s: &str) -> IResult<&str, Label> {
    alpha1.map(String::from).parse(s)
}
pub(crate) fn instruction(s: &str) -> IResult<&str, (Label, Operation)> {
    tuple((label, operation)).parse(s)
}

pub(crate) fn instructions(s: &str) -> IResult<&str, Vec<(Label, Operation)>> {
    separated_list1(char(','), instruction).parse(s)
}
