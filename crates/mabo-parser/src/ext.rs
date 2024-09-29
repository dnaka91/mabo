#![expect(clippy::inline_always)]

use std::marker::PhantomData;

use winnow::{stream::Location, PResult, Parser};

pub(crate) trait ParserExt {
    fn map_err<G, I, O, E, E2>(self, map: G) -> MapErr<Self, G, I, O, E, E2>
    where
        G: Fn(E) -> E2,
        Self: Parser<I, O, E> + Sized;

    fn map_err_loc<G, I, O, E, E2>(self, map: G) -> MapErrLoc<Self, G, I, O, E, E2>
    where
        G: Fn(usize, E) -> E2,
        Self: Parser<I, O, E> + Sized;
}

impl<T> ParserExt for T {
    #[inline(always)]
    fn map_err<G, I, O, E, E2>(self, map: G) -> MapErr<Self, G, I, O, E, E2>
    where
        G: Fn(E) -> E2,
        Self: Parser<I, O, E> + Sized,
    {
        MapErr::new(self, map)
    }

    #[inline(always)]
    fn map_err_loc<G, I, O, E, E2>(self, map: G) -> MapErrLoc<Self, G, I, O, E, E2>
    where
        G: Fn(usize, E) -> E2,
        Self: Parser<I, O, E> + Sized,
    {
        MapErrLoc::new(self, map)
    }
}

pub(crate) struct MapErr<F, G, I, O, E, E2>
where
    F: Parser<I, O, E>,
    G: Fn(E) -> E2,
{
    parser: F,
    map: G,
    i: PhantomData<I>,
    o: PhantomData<O>,
    e: PhantomData<E>,
    e2: PhantomData<E2>,
}

impl<F, G, I, O, E, E2> MapErr<F, G, I, O, E, E2>
where
    F: Parser<I, O, E>,
    G: Fn(E) -> E2,
{
    #[inline(always)]
    pub(crate) fn new(parser: F, map: G) -> Self {
        Self {
            parser,
            map,
            i: PhantomData,
            o: PhantomData,
            e: PhantomData,
            e2: PhantomData,
        }
    }
}

impl<F, G, I, O, E, E2> Parser<I, O, E2> for MapErr<F, G, I, O, E, E2>
where
    F: Parser<I, O, E>,
    G: Fn(E) -> E2,
{
    #[inline]
    fn parse_next(&mut self, i: &mut I) -> PResult<O, E2> {
        match self.parser.parse_next(i) {
            Ok(o) => Ok(o),
            Err(e) => Err(e.map(|e| (self.map)(e))),
        }
    }
}

pub(crate) struct MapErrLoc<F, G, I, O, E, E2>
where
    F: Parser<I, O, E>,
    G: Fn(usize, E) -> E2,
{
    parser: F,
    map: G,
    i: PhantomData<I>,
    o: PhantomData<O>,
    e: PhantomData<E>,
    e2: PhantomData<E2>,
}

impl<F, G, I, O, E, E2> MapErrLoc<F, G, I, O, E, E2>
where
    F: Parser<I, O, E>,
    G: Fn(usize, E) -> E2,
{
    #[inline(always)]
    pub(crate) fn new(parser: F, map: G) -> Self {
        Self {
            parser,
            map,
            i: PhantomData,
            o: PhantomData,
            e: PhantomData,
            e2: PhantomData,
        }
    }
}

impl<F, G, I, O, E, E2> Parser<I, O, E2> for MapErrLoc<F, G, I, O, E, E2>
where
    F: Parser<I, O, E>,
    G: Fn(usize, E) -> E2,
    I: Location,
{
    #[inline]
    fn parse_next(&mut self, i: &mut I) -> PResult<O, E2> {
        match self.parser.parse_next(i) {
            Ok(o) => Ok(o),
            Err(e) => Err(e.map(|e| (self.map)(i.location(), e))),
        }
    }
}
