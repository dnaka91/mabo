use std::ops::Range;

use winnow::{
    stream::{Checkpoint, Location, Offset, Stream},
    Located,
};

pub fn from_until<const N: usize, I>(
    mut input: Located<I>,
    start: Checkpoint<<I as Stream>::Checkpoint>,
    chars: [char; N],
) -> Range<usize>
where
    I: Clone + Offset + Stream<Token = char>,
{
    input.reset(start);

    let start = input.location();

    let end = chars
        .into_iter()
        .find_map(|target| input.offset_for(|c| c == target))
        .map(|pos| pos + 1);

    start..end.unwrap_or(start)
}
