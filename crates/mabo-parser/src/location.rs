use std::ops::Range;

use winnow::{
    LocatingSlice,
    stream::{Checkpoint, Location, Offset, Stream},
};

pub fn from_until<const N: usize, I>(
    mut input: LocatingSlice<I>,
    start: &Checkpoint<<I as Stream>::Checkpoint, LocatingSlice<I>>,
    chars: [char; N],
) -> Range<usize>
where
    I: Clone + Offset + Stream<Token = char>,
{
    input.reset(start);

    let start = input.current_token_start();

    let end = chars
        .into_iter()
        .find_map(|target| input.offset_for(|c| c == target))
        .map(|pos| pos + 1);

    start..end.unwrap_or(start)
}
