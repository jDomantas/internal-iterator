use crate::{InternalIterator, IntoInternalIterator};


/// An iterator that links two iterators together, in a chain.
pub struct Chain<A, B> {
    pub(crate) first: A,
    pub(crate) second: B,
}

impl<A, B> InternalIterator for Chain<A, B>
where
    A: InternalIterator,
    B: InternalIterator<Item = A::Item>,
{
    type Item = A::Item;

    fn find_map<R, C>(self, mut consumer: C) -> Option<R>
    where
        C: FnMut(Self::Item) -> Option<R>
    {
        let Self { first, second } = self;
        first.find_map(&mut consumer).or_else(|| second.find_map(consumer))
    }
}


/// An iterator that clones the elements of an underlying iterator.
pub struct Cloned<I> {
    pub(crate) iter: I,
}

impl<'a, I, T: 'a> InternalIterator for Cloned<I>
where
    I: InternalIterator<Item = &'a T>,
    T: Clone,
{
    type Item = T;

    fn find_map<R, C>(self, mut consumer: C) -> Option<R>
    where
        C: FnMut(Self::Item) -> Option<R>
    {
        self.iter.find_map(|item| consumer(item.clone()))
    }
}


/// An iterator that copies the elements of an underlying iterator.
pub struct Copied<I> {
    pub(crate) iter: I,
}

impl<'a, I, T: 'a> InternalIterator for Copied<I>
where
    I: InternalIterator<Item = &'a T>,
    T: Copy,
{
    type Item = T;

    fn find_map<R, C>(self, mut consumer: C) -> Option<R>
    where
        C: FnMut(Self::Item) -> Option<R>
    {
        self.iter.find_map(|&item| consumer(item))
    }
}


/// An iterator that yields the current count and the element during iteration.
pub struct Enumerate<I> {
    pub(crate) iter: I,
}

impl<I> InternalIterator for Enumerate<I>
where
    I: InternalIterator,
{
    type Item = (usize, I::Item);

    fn find_map<R, C>(self, mut consumer: C) -> Option<R>
    where
        C: FnMut(Self::Item) -> Option<R>
    {
        let mut idx = 0;
        self.iter.find_map(|item| {
            let next = idx + 1;
            let idx = core::mem::replace(&mut idx, next);
            consumer((idx, item))
        })
    }
}


/// An iterator that filters the elements of `iter` with `predicate`.
pub struct Filter<I, F> {
    pub(crate) iter: I,
    pub(crate) predicate: F,
}

impl<I, F> InternalIterator for Filter<I, F>
where
    I: InternalIterator,
    F: FnMut(&I::Item) -> bool,
{
    type Item = I::Item;

    fn find_map<R, C>(self, mut consumer: C) -> Option<R>
    where
        C: FnMut(Self::Item) -> Option<R>
    {
        let Self { iter, mut predicate } = self;
        iter.find_map(|item| {
            if predicate(&item) {
                consumer(item)
            } else {
                None
            }
        })
    }
}


/// An iterator that uses `f` to both filter and map elements from `iter`.
pub struct FilterMap<I, F> {
    pub(crate) iter: I,
    pub(crate) f: F,
}

impl<I, F, T> InternalIterator for FilterMap<I, F>
where
    I: InternalIterator,
    F: FnMut(I::Item) -> Option<T>,
{
    type Item = T;

    fn find_map<R, C>(self, mut consumer: C) -> Option<R>
    where
        C: FnMut(Self::Item) -> Option<R>
    {
        let Self { iter, mut f } = self;
        iter.find_map(|item| f(item).and_then(&mut consumer))
    }
}


/// An iterator that maps each element to an iterator, and yields the elements
/// of the produced iterators.
pub struct FlatMap<I, F> {
    pub(crate) iter: I,
    pub(crate) f: F,
}

impl<I, F, T, U> InternalIterator for FlatMap<I, F>
where
    I: InternalIterator,
    F: FnMut(I::Item) -> U,
    U: IntoInternalIterator<Item = T>,
{
    type Item = T;

    fn find_map<R, C>(self, mut consumer: C) -> Option<R>
    where
        C: FnMut(Self::Item) -> Option<R>
    {
        let Self { iter, mut f } = self;
        iter.find_map(|item| f(item).into_internal_iter().find_map(&mut consumer))
    }
}


/// An iterator that calls a function with a reference to each element before
/// yielding it.
pub struct Inspect<I, F> {
    pub(crate) iter: I,
    pub(crate) f: F,
}

impl<I, F> InternalIterator for Inspect<I, F>
where
    I: InternalIterator,
    F: FnMut(&I::Item),
{
    type Item = I::Item;

    fn find_map<R, C>(self, mut consumer: C) -> Option<R>
    where
        C: FnMut(Self::Item) -> Option<R>
    {
        let Self { iter, mut f } = self;
        iter.find_map(|item| {
            f(&item);
            consumer(item)
        })
    }
}


/// An iterator that maps the values of `iter` with `f`.
pub struct Map<I, F> {
    pub(crate) iter: I,
    pub(crate) f: F,
}

impl<I, F, T> InternalIterator for Map<I, F>
where
    I: InternalIterator,
    F: FnMut(I::Item) -> T,
{
    type Item = T;

    fn find_map<R, C>(self, mut consumer: C) -> Option<R>
    where
        C: FnMut(Self::Item) -> Option<R>
    {
        let Self { iter, mut f } = self;
        iter.find_map(|item| consumer(f(item)))
    }
}


/// An iterator that skips over `n` elements of `iter`.
pub struct Skip<I> {
    pub(crate) iter: I,
    pub(crate) n: usize,
}

impl<I> InternalIterator for Skip<I>
where
    I: InternalIterator,
{
    type Item = I::Item;

    fn find_map<R, C>(self, mut consumer: C) -> Option<R>
    where
        C: FnMut(Self::Item) -> Option<R>
    {
        let Self { iter, mut n } = self;
        iter.find_map(|item| {
            if n == 0 {
                consumer(item)
            } else {
                n -= 1;
                None
            }
        })
    }
}


/// An iterator that only iterates over the first `n` iterations of `iter`.
pub struct Take<I> {
    pub(crate) iter: I,
    pub(crate) n: usize,
}

impl<I> InternalIterator for Take<I>
where
    I: InternalIterator,
{
    type Item = I::Item;

    fn find_map<R, C>(self, mut consumer: C) -> Option<R>
    where
        C: FnMut(Self::Item) -> Option<R>
    {
        let Self { iter, mut n } = self;
        if n == 0 {
            return None;
        }
        iter.find_map(|item| {
            if n > 0 {
                n -= 1;
                let result = consumer(item);
                if n == 0 || result.is_some() {
                    Some(result)
                } else {
                    None
                }
            } else {
                Some(None)
            }
        }).unwrap_or(None)
    }
}


/// A wrapper type to convert [`std::iter::Iterator`] to [`InternalIterator`].
pub struct Internal<I> {
    pub(crate) iterator: I,
}

impl<I> InternalIterator for Internal<I>
where
    I: Iterator
{
    type Item = I::Item;

    fn find_map<T, F>(mut self, consumer: F) -> Option<T>
    where
        F: FnMut(Self::Item) -> Option<T>
    {
        self.iterator.find_map(consumer)
    }
}
