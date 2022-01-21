use crate::lib::wordle::{CompareResult, State, Word};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while_m_n},
    character::complete::{line_ending, multispace0, space0, space1},
    combinator::{all_consuming, map, value},
    multi::{fill, separated_list0},
    sequence::{delimited, separated_pair, terminated},
    IResult,
};

pub fn parse_board(input: &str) -> IResult<&str, Vec<(Word, CompareResult)>> {
    let word = map(
        take_while_m_n(5, 5, |x: char| x.is_ascii_alphabetic()),
        |xs: &str| xs.parse::<Word>().unwrap(),
    );

    let entry = separated_pair(word, space1, compare_result);
    let line = delimited(space0, entry, space0);
    let board = terminated(separated_list0(line_ending, line), multispace0);
    all_consuming(board)(input)
}

fn compare_result(input: &str) -> IResult<&str, CompareResult> {
    let state = |i| {
        alt((
            value(State::CorrectLocation, tag("g")),
            value(State::IncorrectLocaiton, tag("y")),
            value(State::NotExists, tag("b")),
        ))(i)
    };
    let mut buf = [State::NotExists; 5];
    let (rest, _) = fill(state, buf.as_mut_slice())(input)?;
    Ok((rest, buf))
}
