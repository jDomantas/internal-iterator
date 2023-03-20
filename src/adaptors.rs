use core::ops::ControlFlow;

use crate::{InternalIterator, IntoInternalIterator};


/// An iterator that links two iterators together, in a chain.
#[derive(Clone)]
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

    fn try_for_each<R, C>(self, mut consumer: C) -> ControlFlow<R>
    where
        C: FnMut(Self::Item) -> ControlFlow<R>
    {
        let Self { first, second } = self;
        match first.try_for_each(&mut consumer) {
            ControlFlow::Continue(()) => second.try_for_each(consumer),
            br => br,
        }
    }

    fn last(self) -> Option<Self::Item> {
        match (self.first.last(), self.second.last()) {
            (_, Some(x)) | (Some(x), None) => Some(x),
            (None, None) => None,
        }
    }
}


/// An iterator that clones the elements of an underlying iterator.
#[derive(Clone)]
pub struct Cloned<I> {
    pub(crate) iter: I,
}

impl<'a, I, T: 'a> InternalIterator for Cloned<I>
where
    I: InternalIterator<Item = &'a T>,
    T: Clone,
{
    type Item = T;

    fn try_for_each<R, C>(self, mut consumer: C) -> ControlFlow<R>
    where
        C: FnMut(Self::Item) -> ControlFlow<R>
    {
        self.iter.try_for_each(|item| consumer(item.clone()))
    }
}


/// An iterator that copies the elements of an underlying iterator.
#[derive(Clone)]
pub struct Copied<I> {
    pub(crate) iter: I,
}

impl<'a, I, T: 'a> InternalIterator for Copied<I>
where
    I: InternalIterator<Item = &'a T>,
    T: Copy,
{
    type Item = T;

    fn try_for_each<R, C>(self, mut consumer: C) -> ControlFlow<R>
    where
        C: FnMut(Self::Item) -> ControlFlow<R>
    {
        self.iter.try_for_each(|&item| consumer(item))
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn last(self) -> Option<Self::Item> {
        self.iter.last().copied()
    }

    fn nth(self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).copied()
    }
}


/// An iterator that yields the current count and the element during iteration.
#[derive(Clone)]
pub struct Enumerate<I> {
    pub(crate) iter: I,
}

impl<I> InternalIterator for Enumerate<I>
where
    I: InternalIterator,
{
    type Item = (usize, I::Item);

    fn try_for_each<R, C>(self, mut consumer: C) -> ControlFlow<R>
    where
        C: FnMut(Self::Item) -> ControlFlow<R>
    {
        let mut idx = 0;
        self.iter.try_for_each(|item| {
            let next = idx + 1;
            let idx = core::mem::replace(&mut idx, next);
            consumer((idx, item))
        })
    }

    fn count(self) -> usize {
        self.iter.count()
    }

    fn nth(self, n: usize) -> Option<Self::Item> {
        let value = self.iter.nth(n)?;
        Some((n, value))
    }
}


/// An iterator that filters the elements of `iter` with `predicate`.
#[derive(Clone)]
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

    fn try_for_each<R, C>(self, mut consumer: C) -> ControlFlow<R>
    where
        C: FnMut(Self::Item) -> ControlFlow<R>
    {
        let Self { iter, mut predicate } = self;
        iter.try_for_each(|item| {
            if predicate(&item) {
                consumer(item)
            } else {
                ControlFlow::Continue(())
            }
        })
    }
}


/// An iterator that uses `f` to both filter and map elements from `iter`.
#[derive(Clone)]
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

    fn try_for_each<R, C>(self, mut consumer: C) -> ControlFlow<R>
    where
        C: FnMut(Self::Item) -> ControlFlow<R>
    {
        let Self { iter, mut f } = self;
        iter.try_for_each(|item| match f(item) {
            Some(mapped) => consumer(mapped),
            None => ControlFlow::Continue(()),
        })
    }
}


/// An iterator that maps each element to an iterator, and yields the elements
/// of the produced iterators.
#[derive(Clone)]
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

    fn try_for_each<R, C>(self, mut consumer: C) -> ControlFlow<R>
    where
        C: FnMut(Self::Item) -> ControlFlow<R>
    {
        let Self { iter, mut f } = self;
        iter.try_for_each(|item| f(item).into_internal_iter().try_for_each(&mut consumer))
    }
}


/// An iterator that calls a function with a reference to each element before
/// yielding it.
#[derive(Clone)]
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

    fn try_for_each<R, C>(self, mut consumer: C) -> ControlFlow<R>
    where
        C: FnMut(Self::Item) -> ControlFlow<R>
    {
        let Self { iter, mut f } = self;
        iter.try_for_each(|item| {
            f(&item);
            consumer(item)
        })
    }
}


/// An iterator that maps the values of `iter` with `f`.
#[derive(Clone)]
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

    fn try_for_each<R, C>(self, mut consumer: C) -> ControlFlow<R>
    where
        C: FnMut(Self::Item) -> ControlFlow<R>
    {
        let Self { iter, mut f } = self;
        iter.try_for_each(|item| consumer(f(item)))
    }
}


/// An iterator that skips over `n` elements of `iter`.
#[derive(Clone)]
pub struct Skip<I> {
    pub(crate) iter: I,
    pub(crate) n: usize,
}

impl<I> InternalIterator for Skip<I>
where
    I: InternalIterator,
{
    type Item = I::Item;

    fn try_for_each<R, C>(self, mut consumer: C) -> ControlFlow<R>
    where
        C: FnMut(Self::Item) -> ControlFlow<R>
    {
        let Self { iter, mut n } = self;
        iter.try_for_each(|item| {
            if n == 0 {
                consumer(item)
            } else {
                n -= 1;
                ControlFlow::Continue(())
            }
        })
    }
}


/// An iterator that only iterates over the first `n` iterations of `iter`.
#[derive(Clone)]
pub struct Take<I> {
    pub(crate) iter: I,
    pub(crate) n: usize,
}

impl<I> InternalIterator for Take<I>
where
    I: InternalIterator,
{
    type Item = I::Item;

    fn try_for_each<R, C>(self, mut consumer: C) -> ControlFlow<R>
    where
        C: FnMut(Self::Item) -> ControlFlow<R>
    {
        let Self { iter, mut n } = self;
        if n == 0 {
            return ControlFlow::Continue(());
        }
        let result = iter.try_for_each(|item| {
            n -= 1;
            match consumer(item) {
                _ if n == 0 => ControlFlow::Break(ControlFlow::Continue(())),
                ControlFlow::Continue(()) => ControlFlow::Continue(()),
                ControlFlow::Break(value) => ControlFlow::Break(ControlFlow::Break(value)),
            }
        });
        match result {
            ControlFlow::Continue(()) => ControlFlow::Continue(()),
            ControlFlow::Break(x) => x,
        }
    }
}


/// A wrapper type to convert [`std::iter::Iterator`] to [`InternalIterator`].
#[derive(Clone)]
pub struct Internal<I> {
    pub(crate) iterator: I,
}

impl<I> InternalIterator for Internal<I>
where
    I: Iterator
{
    type Item = I::Item;

    fn try_for_each<T, F>(mut self, consumer: F) -> ControlFlow<T>
    where
        F: FnMut(Self::Item) -> ControlFlow<T>
    {
        self.iterator.try_for_each(consumer)
    }

    fn count(self) -> usize {
        self.iterator.count()
    }

    fn last(self) -> Option<Self::Item> {
        self.iterator.last()
    }

    fn nth(mut self, n: usize) -> Option<Self::Item> {
        self.iterator.nth(n)
    }
}
