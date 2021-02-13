use crate::{InternalIterator, IntoInternalIterator};


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

    fn find_map<C, R>(self, mut consumer: C) -> Option<R>
    where
        C: FnMut(Self::Item) -> Option<R>
    {
        let Self { first, second } = self;
        first.find_map(&mut consumer).or_else(|| second.find_map(consumer))
    }
}


pub struct Cloned<I> {
    pub(crate) iter: I,
}

impl<'a, I, T: 'a> InternalIterator for Cloned<I>
where
    I: InternalIterator<Item = &'a T>,
    T: Clone,
{
    type Item = T;

    fn find_map<C, R>(self, mut consumer: C) -> Option<R>
    where
        C: FnMut(Self::Item) -> Option<R>
    {
        self.iter.find_map(|item| consumer(item.clone()))
    }
}


pub struct Copied<I> {
    pub(crate) iter: I,
}

impl<'a, I, T: 'a> InternalIterator for Copied<I>
where
    I: InternalIterator<Item = &'a T>,
    T: Copy,
{
    type Item = T;

    fn find_map<C, R>(self, mut consumer: C) -> Option<R>
    where
        C: FnMut(Self::Item) -> Option<R>
    {
        self.iter.find_map(|&item| consumer(item))
    }
}


pub struct Enumerate<I> {
    pub(crate) iter: I,
}

impl<I> InternalIterator for Enumerate<I>
where
    I: InternalIterator,
{
    type Item = (usize, I::Item);

    fn find_map<C, R>(self, mut consumer: C) -> Option<R>
    where
        C: FnMut(Self::Item) -> Option<R>
    {
        let mut idx = 0;
        self.iter.find_map(|item| {
            let next = idx + 1;
            let idx = std::mem::replace(&mut idx, next);
            consumer((idx, item))
        })
    }
}


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

    fn find_map<C, R>(self, mut consumer: C) -> Option<R>
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

    fn find_map<C, R>(self, mut consumer: C) -> Option<R>
    where
        C: FnMut(Self::Item) -> Option<R>
    {
        let Self { iter, mut f } = self;
        iter.find_map(|item| f(item).and_then(&mut consumer))
    }
}


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

    fn find_map<C, R>(self, mut consumer: C) -> Option<R>
    where
        C: FnMut(Self::Item) -> Option<R>
    {
        let Self { iter, mut f } = self;
        iter.find_map(|item| f(item).into_internal_iter().find_map(&mut consumer))
    }
}


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

    fn find_map<C, R>(self, mut consumer: C) -> Option<R>
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

    fn find_map<C, R>(self, mut consumer: C) -> Option<R>
    where
        C: FnMut(Self::Item) -> Option<R>
    {
        let Self { iter, mut f } = self;
        iter.find_map(|item| consumer(f(item)))
    }
}


pub struct Skip<I> {
    pub(crate) iter: I,
    pub(crate) n: usize,
}

impl<I> InternalIterator for Skip<I>
where
    I: InternalIterator,
{
    type Item = I::Item;

    fn find_map<C, R>(self, mut consumer: C) -> Option<R>
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


pub struct Take<I> {
    pub(crate) iter: I,
    pub(crate) n: usize,
}

impl<I> InternalIterator for Take<I>
where
    I: InternalIterator,
{
    type Item = I::Item;

    fn find_map<C, R>(self, mut consumer: C) -> Option<R>
    where
        C: FnMut(Self::Item) -> Option<R>
    {
        let Self { iter, mut n } = self;
        iter.find_map(|item| {
            if n > 0 {
                n -= 1;
                consumer(item)
            } else {
                None
            }
        })
    }
}


pub struct Internal<I> {
    pub(crate) iterator: I,
}

impl<I> InternalIterator for Internal<I>
where
    I: Iterator
{
    type Item = I::Item;

    fn find_map<F, T>(mut self, consumer: F) -> Option<T>
    where
        F: FnMut(Self::Item) -> Option<T>
    {
        self.iterator.find_map(consumer)
    }
}
