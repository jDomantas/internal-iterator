pub mod adaptors;
mod std_impls;

use std::cmp::Ordering;
use crate::adaptors::*;

pub trait InternalIterator: Sized {
    type Item;

    /// ```
    /// use internal_iterator::{InternalIterator, IteratorExt};
    ///
    /// let a = ["lol", "two", "NaN", "4", "5"];
    ///
    /// let parsed = a
    ///     .iter()
    ///     .into_internal()
    ///     .find_map(|x| x.parse().ok());
    ///
    /// assert_eq!(parsed, Some(4));
    /// ```
    fn find_map<F, R>(self, f: F) -> Option<R>
    where
        F: FnMut(Self::Item) -> Option<R>;

    /// ```
    /// use internal_iterator::{InternalIterator, IteratorExt};
    ///
    /// let a = [1, 2, 3];
    /// assert!(a.iter().into_internal().all(|&x| x > 0));
    /// assert!(!a.iter().into_internal().all(|&x| x < 2));
    /// ```
    fn all<F>(self, mut f: F) -> bool
    where
        F: FnMut(Self::Item) -> bool,
    {
        self.find_map(|item| if f(item) { None } else { Some(()) }).is_none()
    }

    /// ```
    /// use internal_iterator::{InternalIterator, IteratorExt};
    ///
    /// let a = [1, 2, 3];
    /// assert!(a.iter().into_internal().any(|&x| x == 2));
    /// assert!(!a.iter().into_internal().any(|&x| x > 5));
    /// ```
    fn any<F>(self, mut f: F) -> bool
    where
        F: FnMut(Self::Item) -> bool,
    {
        self.find_map(|item| if f(item) { Some(()) } else { None }).is_some()
    }

    /// ```
    /// use internal_iterator::{InternalIterator, IteratorExt};
    ///
    /// let a1 = [1, 2, 3];
    /// let a2 = [4, 5, 6];
    ///
    /// let chained = a1.iter().into_internal()
    ///     .chain(a2.iter().into_internal())
    ///     .collect::<Vec<_>>();
    ///
    /// assert_eq!(chained, vec![&1, &2, &3, &4, &5, &6]);
    /// ```
    fn chain<U>(self, other: U) -> Chain<Self, <U as IntoInternalIterator>::IntoIter>
    where
        U: IntoInternalIterator<Item = Self::Item>,
    {
        Chain { first: self, second: other.into_internal_iter() }
    }

    /// ```
    /// use internal_iterator::{InternalIterator, IteratorExt};
    ///
    /// let a = [1, 2, 3];
    ///
    /// let cloned = a.iter().into_internal().cloned().collect::<Vec<_>>();
    ///
    /// assert_eq!(cloned, vec![1, 2, 3]);
    /// ```
    fn cloned<'a, T: 'a>(self) -> Cloned<Self>
    where
        Self: InternalIterator<Item = &'a T>,
        T: Clone,
    {
        Cloned { iter: self }
    }

    /// ```
    /// use internal_iterator::{InternalIterator, IteratorExt};
    ///
    /// let a = [1, 2, 3];
    ///
    /// let doubled = a
    ///     .iter()
    ///     .into_internal()
    ///     .map(|&x| x * 2)
    ///     .collect::<Vec<_>>();
    ///
    /// assert_eq!(doubled, vec![2, 4, 6]);
    /// ```
    fn collect<B>(self) -> B
    where
        B: FromInternalIterator<Self::Item>,
    {
        B::from_iter(self)
    }

    /// ```
    /// use internal_iterator::{InternalIterator, IteratorExt};
    ///
    /// let a = [1, 2, 3];
    ///
    /// let cloned = a.iter().into_internal().copied().collect::<Vec<_>>();
    ///
    /// assert_eq!(cloned, vec![1, 2, 3]);
    /// ```
    fn copied<'a, T: 'a>(self) -> Copied<Self>
    where
        Self: InternalIterator<Item = &'a T>,
        T: Copy,
    {
        Copied { iter: self }
    }

    /// ```
    /// use internal_iterator::{InternalIterator, IteratorExt};
    ///
    /// let a = [1, 2, 3];
    ///
    /// assert_eq!(a.iter().into_internal().count(), 3);
    /// ```
    fn count(self) -> usize {
        let mut count = 0;
        self.for_each(|_| count += 1);
        count
    }

    // TODO: cycle

    /// ```
    /// use internal_iterator::{InternalIterator, IteratorExt};
    ///
    /// let a = ['a', 'b', 'c'];
    ///
    /// let enumerated = a.iter().into_internal().enumerate().collect::<Vec<_>>();
    ///
    /// assert_eq!(enumerated, vec![(0, &'a'), (1, &'b'), (2, &'c')]);
    /// ```
    fn enumerate(self) -> Enumerate<Self> {
        Enumerate { iter: self }
    }

    /// ```
    /// use internal_iterator::{InternalIterator, IteratorExt};
    ///
    /// let a = [0i32, 1, 2];
    ///
    /// let positive = a.iter().into_internal().filter(|x| x.is_positive()).collect::<Vec<_>>();
    ///
    /// assert_eq!(positive, vec![&1, &2]);
    /// ```
    fn filter<P>(self, predicate: P) -> Filter<Self, P>
    where
        P: FnMut(&Self::Item) -> bool,
    {
        Filter { iter: self, predicate }
    }

    /// ```
    /// use internal_iterator::{InternalIterator, IteratorExt};
    ///
    /// let a = ["1", "two", "NaN", "four", "5"];
    ///
    /// let parsed: Vec<_> = a
    ///     .iter()
    ///     .into_internal()
    ///     .filter_map(|x| x.parse::<i32>().ok())
    ///     .collect();
    ///
    /// assert_eq!(parsed, vec![1, 5]);
    /// ```
    fn filter_map<T, F>(self, f: F) -> FilterMap<Self, F>
    where
        F: FnMut(Self::Item) -> Option<T>,
    {
        FilterMap { iter: self, f }
    }

    /// ```
    /// use internal_iterator::{InternalIterator, IteratorExt};
    ///
    /// let a = [1, 2, 3];
    ///
    /// assert_eq!(a.iter().into_internal().find(|&&x| x == 2), Some(&2));
    ///
    /// assert_eq!(a.iter().into_internal().find(|&&x| x == 5), None);
    /// ```
    fn find<F>(self, mut f: F) -> Option<Self::Item>
    where
        F: FnMut(&Self::Item) -> bool,
    {
        self.find_map(|item| {
            if f(&item) {
                Some(item)
            } else {
                None
            }
        })
    }

    /// ```
    /// use internal_iterator::{InternalIterator, IteratorExt};
    ///
    /// let a = [1, 2, 3];
    ///
    /// let mapped = a.iter()
    ///     .into_internal()
    ///     .flat_map(|&x| vec![x * 10 + 2, x * 10 + 3].into_iter().into_internal())
    ///     .collect::<Vec<_>>();
    ///
    /// assert_eq!(mapped, vec![12, 13, 22, 23, 32, 33]);
    /// ```
    fn flat_map<U, F>(self, f: F) -> FlatMap<Self, F>
    where
        F: FnMut(Self::Item) -> U,
        U: IntoInternalIterator,
    {
        FlatMap { iter: self, f }
    } 

    // TODO: flatten

    // TODO: fn fold<B, F>(self, init: B, f: F) -> B
    // where
    //     F: FnMut(B, Self::Item) -> B,
    // { }

    fn for_each<F>(self, mut f: F)
    where
        F: FnMut(Self::Item)
    {
        self.find_map::<_, ()>(|item| {
            f(item);
            None
        });
    }

    fn inspect<F>(self, f: F) -> Inspect<Self, F>
    where
        F: FnMut(&Self::Item)
    {
        Inspect { iter: self, f }
    }

    fn last(self) -> Option<Self::Item> {
        let mut last = None;
        self.for_each(|item| last = Some(item));
        last
    }

    /// ```
    /// use internal_iterator::{InternalIterator, IteratorExt};
    ///
    /// let a = [1, 2, 3];
    ///
    /// let doubled = a
    ///     .iter()
    ///     .into_internal()
    ///     .map(|&x| x * 2)
    ///     .collect::<Vec<_>>();
    ///
    /// assert_eq!(doubled, vec![2, 4, 6]);
    /// ```
    fn map<F, T>(self, f: F) -> Map<Self, F>
    where
        F: FnMut(Self::Item) -> T,
    {
        Map { iter: self, f }
    }
    
    fn max(self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        self.max_by(Ord::cmp)
    }
    
    fn max_by<F>(self, mut compare: F) -> Option<Self::Item>
    where
        F: FnMut(&Self::Item, &Self::Item) -> Ordering,
    {
        let mut max = None;
        self.for_each(|item| {
            match max.take() {
                None => max = Some(item),
                Some(i) => {
                    max = Some(max_by(item, i, &mut compare));
                }
            }
        });
        max
    }

    fn max_by_key<B, F>(self, mut f: F) -> Option<Self::Item>
    where
        F: FnMut(&Self::Item) -> B,
        B: Ord,
    {
        let mut max = None;
        self.for_each(|item| {
            match max.take() {
                None => max = Some(item),
                Some(i) => {
                    max = Some(max_by(item, i, |a, b| f(a).cmp(&f(b))));
                }
            }
        });
        max
    }
    
    fn min(self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        self.min_by(Ord::cmp)
    }
    
    fn min_by<F>(self, mut compare: F) -> Option<Self::Item>
    where
        F: FnMut(&Self::Item, &Self::Item) -> Ordering,
    {
        let mut min = None;
        self.for_each(|item| {
            match min.take() {
                None => min = Some(item),
                Some(i) => {
                    min = Some(min_by(item, i, &mut compare));
                }
            }
        });
        min
    }
    
    fn min_by_key<B, F>(self, mut f: F) -> Option<Self::Item>
    where
        F: FnMut(&Self::Item) -> B,
        B: Ord,
    {
        let mut min = None;
        self.for_each(|item| {
            match min.take() {
                None => min = Some(item),
                Some(i) => {
                    min = Some(min_by(item, i, |a, b| f(a).cmp(&f(b))));
                }
            }
        });
        min
    }
    
    fn nth(self, mut n: usize) -> Option<Self::Item> {
        self.find_map(|item| {
            if n == 0 {
                Some(item)
            } else {
                n -= 1;
                None
            }
        })
    }
    
    fn partition<B, F>(self, mut f: F) -> (B, B)
    where
        F: FnMut(&Self::Item) -> bool,
        B: Default + Extend<Self::Item>,
    {
        let mut trues = B::default();
        let mut falses = B::default();
        self.for_each(|item| {
            if f(&item) {
                trues.extend(std::iter::once(item));
            } else {
                falses.extend(std::iter::once(item));
            }
        });
        (trues, falses)
    }
    
    fn position<F>(self, mut f: F) -> Option<usize>
    where
        F: FnMut(Self::Item) -> bool,
    {
        self.enumerate().find_map(|(idx, item)| {
            if f(item) {
                Some(idx)
            } else {
                None
            }
        })
    }
    
    // TODO: product

    // TODO: scan

    fn skip(self, n: usize) -> Skip<Self> {
        Skip { iter: self, n }
    }

    // TODO: skip_while

    // TODO: step_by

    // TODO: sum

    fn take(self, n: usize) -> Take<Self> {
        Take { iter: self, n }
    }

    // TODO: take_while

    // TODO: try_find

    // TODO: try_fold

    // TODO: try_for_each

    // TODO: unzip
}

pub trait IntoInternalIterator {
    type Item;
    type IntoIter: InternalIterator<Item = Self::Item>;

    fn into_internal_iter(self) -> Self::IntoIter;
}

impl<I> IntoInternalIterator for I
where
    I: InternalIterator,
{
    type Item = I::Item;

    type IntoIter = I;

    fn into_internal_iter(self) -> Self::IntoIter {
        self
    }
}

/// Extension trait to add conversion to [`InternalIterator`] for regular
/// iterators.
pub trait IteratorExt: IntoIterator {
    /// Convert an [`std::iter::Iterator`] to an [`InternalIterator`].
    ///
    /// Composing internal iterators together requires all used iterators to be
    /// internal iterators. Given that regular iterators are far more prevalent,
    /// this function can be used to allow them to be used together with
    /// internal iterators.
    ///
    /// ```
    /// # use internal_iterator::InternalIterator;
    /// use internal_iterator::IteratorExt;
    ///
    /// fn flatten_ranges(ranges: impl InternalIterator<Item = (i32, i32)>) -> impl InternalIterator<Item = i32> {
    ///     ranges.flat_map(|(from, to)| (from..to).into_internal())
    /// }
    fn into_internal(self) -> Internal<Self::IntoIter>
    where
        Self: Sized,
    {
        Internal { iterator: self.into_iter() }
    }
}

impl<I: IntoIterator> IteratorExt for I {}

/// Conversion from an `InternalIterator`.
///
/// This is internal-iterator equivalent of [`std::iter::FromIterator`].
pub trait FromInternalIterator<A> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoInternalIterator<Item = A>;
}

fn max_by<A, C: FnMut(&A, &A) -> Ordering>(x: A, y: A, mut compare: C) -> A {
    match compare(&x, &y) {
        Ordering::Less => y,
        Ordering::Equal |
        Ordering::Greater => x,
    }
}

fn min_by<A, C: FnMut(&A, &A) -> Ordering>(x: A, y: A, mut compare: C) -> A {
    match compare(&x, &y) {
        Ordering::Less |
        Ordering::Equal => x,
        Ordering::Greater => y,
    }
}
