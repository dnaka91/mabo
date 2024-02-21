//! Punctionation logic for many Mabo schema elements. The main point of interest is the
//! [`Punctuated`] container.

use std::{
    fmt::{self, Write},
    iter, slice,
};

use crate::{
    token::{self, Delimiter, Punctuation},
    Print,
};

/// Container for a list of elements that are separated by punctuation.
///
/// This structure is never empty, always holding at least one element. All values must be separated
/// by punctuation and the last element can have an optional trailing punctuation.
///
/// The punctuation defaults to a comma `,` as this is the most common one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Punctuated<T, P = token::Comma> {
    values: Vec<(T, P)>,
    last: Box<(T, Option<P>)>,
}

impl<T, P> Punctuated<T, P> {
    /// Construct a new `Punctuated<T, P>` from the given arguments.
    pub fn new(values: Vec<(T, P)>, last: (T, Option<P>)) -> Self {
        Self {
            values,
            last: Box::new(last),
        }
    }

    /// Returns an iterator over the values (excluding the punctuation).
    #[must_use]
    pub fn values(&self) -> ValuesIter<'_, T, P> {
        ValuesIter {
            items: self.values.iter(),
            last: iter::once(&self.last.0),
        }
    }

    /// Returns the number of elements.
    ///
    /// **Note:** There is no `is_empty` method because this type always carries at least one
    /// element and can never be empty.
    #[allow(clippy::len_without_is_empty)]
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len() + 1
    }
}

impl<T, P: Copy> Punctuated<T, P> {
    /// Returns an iterator over the elements.
    #[must_use]
    pub fn iter(&self) -> Iter<'_, T, P> {
        Iter {
            items: self.values.iter(),
            last: iter::once(&self.last),
        }
    }
}

impl<T: Print, P: Punctuation> Punctuated<T, P> {
    pub(crate) fn surround<D: Delimiter>(
        &self,
        f: &mut fmt::Formatter<'_>,
        level: usize,
        newline: bool,
    ) -> fmt::Result {
        f.write_char(D::OPEN)?;
        if newline {
            f.write_char('\n')?;
        }

        for (value, _) in &self.values {
            value.print(f, level + 1)?;
            f.write_str(P::VALUE)?;
            f.write_char(if newline { '\n' } else { ' ' })?;
        }

        self.last.0.print(f, level + 1)?;
        if self.last.1.is_some() {
            f.write_str(P::VALUE)?;
        }
        if newline {
            f.write_char('\n')?;
        }

        T::indent(f, level)?;
        f.write_char(D::CLOSE)
    }
}

impl<'a, T, P: Copy> IntoIterator for &'a Punctuated<T, P> {
    type IntoIter = Iter<'a, T, P>;
    type Item = (&'a T, Option<P>);

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over the entries of a [`Punctuated`].
///
/// This `struct` is created by the [`iter`](Punctuated::iter) method on [`Punctuated`]. See
/// its documentation for more.
pub struct Iter<'a, T, P = token::Comma> {
    items: slice::Iter<'a, (T, P)>,
    last: iter::Once<&'a (T, Option<P>)>,
}

impl<'a, T, P: Copy> Iterator for Iter<'a, T, P> {
    type Item = (&'a T, Option<P>);

    fn next(&mut self) -> Option<Self::Item> {
        self.items
            .next()
            .map(|(t, p)| (t, Some(*p)))
            .or_else(|| self.last.next().map(|(t, p)| (t, *p)))
    }
}

impl<T, P: Copy> ExactSizeIterator for Iter<'_, T, P> {
    fn len(&self) -> usize {
        self.items.len() + 1
    }
}

/// An iterator over the values of a [`Punctuated`].
///
/// This `struct` is created by the [`values`](Punctuated::values) method on [`Punctuated`]. See
/// its documentation for more.
pub struct ValuesIter<'a, T, P = token::Comma> {
    #[allow(clippy::type_complexity)]
    items: slice::Iter<'a, (T, P)>,
    last: iter::Once<&'a T>,
}

impl<'a, T, P> Iterator for ValuesIter<'a, T, P> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.items
            .next()
            .map(|(t, _)| t)
            .or_else(|| self.last.next())
    }
}

impl<T, P> ExactSizeIterator for ValuesIter<'_, T, P> {
    fn len(&self) -> usize {
        self.items.len() + 1
    }
}
