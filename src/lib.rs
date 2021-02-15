#![doc = "Internal iterator equivalent of [`std::iter::Iterator`].

In some cases implementing `Iterator` can be difficult - for tree shaped
structures you would need to store iteration state at every level, which
implies dynamic allocation and nontrivial amounts of state. On the other
hand, internal iteration is roughly equivalent to calling a provided
function on every element you need to yield and is much simpler to
implement.

This library provides to provide `std`-like iteration facilities, but
based on internal iteration. The goal is to be easy to make use of and feel
familiar to users of `Iterator`. There is one core trait, [`InternalIterator`].
By implementing it you can use its provided methods to construct iterator
pipelines similar to those possible by using regular iterators.

# Implementing `InternalIterator`

Whereas the driving method for regular iterators is [`Iterator::next`], the one
used here is [`InternalIterator::find_map`].

```rust
use internal_iterator::{InternalIterator, IteratorExt};

struct Tree {
    value: i32,
    children: Vec<Tree>,
}

// We implement InternalIterator on the tree directly. You could also
// introduce a wrapper struct and create it in `.iter()`, `.iter_mut()`, just
// like with usual collections.
impl InternalIterator for Tree {
    type Item = i32;

    fn find_map<T, F>(self, mut f: F) -> Option<T>
    where
        F: FnMut(i32) -> Option<T>,
    {
        self.find_map_helper(&mut f)
    }
}

impl Tree {
    fn find_map_helper<T>(&self, f: &mut impl FnMut(i32) -> Option<T>) -> Option<T> {
        let result = f(self.value);
        if result.is_some() {
            return result;
        }
        for child in &self.children {
            let result = child.find_map_helper(f);
            if result.is_some() {
                return result;
            }
        }
        None
    }
}

// now we can use InternalIterator facilities to construct iterator pipelines

let tree = Tree {
    value: 1,
    children: vec![
        Tree {
            value: 2,
            children: Vec::new(),
        },
        Tree {
            value: 3,
            children: vec![
                Tree {
                    value: 4,
                    children: Vec::new(),
                },
            ]
        },
    ]
};

let result = tree
    .map(|x| x * 2)
    .filter(|&x| x > 3)
    .flat_map(|x| vec![x, x * 10]
        .into_iter()
        .into_internal())
    .collect::<Vec<_>>();

assert_eq!(result, vec![4, 40, 6, 60, 8, 80]);
```

# Differences from `std::iter::Iterator`

The main difference between `Iterator` and `InternalIterator` traits is that
all methods in `InternalIterator` consume the iterators. While for regular
iterators you can for example call `nth` and then keep using the iterator with
remaining elements being untouched, you cannot do so with `InternalIterator`.
This is a deliberate choice, as the goal of this library allow having a simpler
iterator implementation without losing too much power. Regular iterators must
keep state to be able to implement `next`, but state is not a hard requirement
for internal iteration and requiring it would defeat the purpose of the library.

Because internal iterators drive themselves instead of being driven by an
outside called, some methods from `Iterator` are not possible to implement. The
most prominent example is [`Iterator::zip`].

# `nostd` compatibility

This crate has two optional features:

* `alloc` - includes `FromIterator` impls for `String`, `Vec`, `BTreeMap`, and
`BTreeSet`. Brings in a dependency on `alloc`.
* `std` - includes `FromIterator` impls for `HashSet` and `HashMap`. Brings in
a dependency on `std`.

Both of these features are enabled by default, but you can disable them if you
are compiling without `std` or even without `alloc`."]

#![cfg_attr(not(feature = "std"), no_std)]

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod adaptors;

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
mod alloc_impls;

#[cfg(feature = "std")]
mod std_impls;

#[cfg(test)]
mod tests;

use core::cmp::Ordering;
pub use crate::adaptors::*;

/// Internal iterator over a collection.
#[must_use = "internal iterators are lazy and do nothing unless consumed"]
pub trait InternalIterator: Sized {
    /// Type of items yielded by the iterator.
    type Item;

    /// Applies function to the elements of iterator and returns the first
    /// non-none result.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
    /// let a = ["lol", "two", "NaN", "4", "5"];
    ///
    /// let parsed = a
    ///     .iter()
    ///     .into_internal()
    ///     .find_map(|x| x.parse().ok());
    ///
    /// assert_eq!(parsed, Some(4));
    /// ```
    fn find_map<R, F>(self, f: F) -> Option<R>
    where
        F: FnMut(Self::Item) -> Option<R>;

    /// Tests if every element of the iterator matches the predicate.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
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

    /// Tests if any element of the iterator matches the predicate.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
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

    /// Takes two iterators and returns an iterator that first iterates over the
    /// elements of the first iterator, and then over the second one.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
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

    /// Creates an iterator yields cloned elements of the original iterator.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
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

    /// Transforms the iterator into a collection.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
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

    /// Creates an iterator yields copied elements of the original iterator.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
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

    /// Returns the number of elements yielded by the iterator.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
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

    /// Creates an iterator that adds the index to every value of the original
    /// iterator.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
    /// let a = ['a', 'b', 'c'];
    ///
    /// let enumerated = a.iter().into_internal().enumerate().collect::<Vec<_>>();
    ///
    /// assert_eq!(enumerated, vec![(0, &'a'), (1, &'b'), (2, &'c')]);
    /// ```
    fn enumerate(self) -> Enumerate<Self> {
        Enumerate { iter: self }
    }

    /// Creates an iterator which only yields elements matching the predicate.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
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

    /// A combination of [`InternalIterator::filter`] and
    /// [`InternalIterator::map`].
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
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

    /// Returns the first element of the iterator that matches the predicate.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
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

    /// Creates and iterator which maps over the elements and flattens the
    /// resulting structure.
    ///
    /// The provided closure is expected to return a type implementing
    /// [`IntoInternalIterator`]. The usual types that work with
    /// [`std::iter::Iterator::flat_map`] don't work here, so you will need to
    /// use [`IteratorExt::into_internal`] to use regular iterators with this
    /// function.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
    /// let a = [1, 2, 3];
    ///
    /// let mapped = a.iter()
    ///     .into_internal()
    ///     .flat_map(|&x| vec![x * 10 + 2, x * 10 + 3]
    ///         .into_iter()
    ///         .into_internal())
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

    /// Run the closure on each element.
    fn for_each<F>(self, mut f: F)
    where
        F: FnMut(Self::Item)
    {
        self.find_map::<(), _>(|item| {
            f(item);
            None
        });
    }

    /// Run the closure on each element, while passing that element on.
    ///
    /// This can be used to inspect the values passed through the iterator
    /// while not modifying the rest of the iterator pipeline.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
    /// let a = [1, 4, 6, 3, 2];
    ///
    /// let v = a.iter()
    ///     .into_internal()
    ///     .filter(|&x| x % 2 == 0)
    ///     .inspect(|x| println!("item: {}", x))
    ///     .map(|x| x / 2)
    ///     .collect::<Vec<_>>();
    ///
    /// assert_eq!(v, vec![2, 3, 1]);
    /// // also prints to stdout:
    /// // item: 4
    /// // item: 6
    /// // item: 2
    /// ```
    fn inspect<F>(self, f: F) -> Inspect<Self, F>
    where
        F: FnMut(&Self::Item)
    {
        Inspect { iter: self, f }
    }

    /// Returns the last element.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
    /// let a = [1, 2, 3];
    /// assert_eq!(a.iter().into_internal().last(), Some(&3));
    ///
    /// let a = [1, 2, 3, 4, 5];
    /// assert_eq!(a.iter().into_internal().last(), Some(&5));
    /// ```
    fn last(self) -> Option<Self::Item> {
        let mut last = None;
        self.for_each(|item| last = Some(item));
        last
    }

    /// Transform each element in the iterator.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
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

    /// Returns the maximum element of an iterator.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
    /// let a = [1, 2, 3];
    /// let b: Vec<u32> = Vec::new();
    ///
    /// assert_eq!(a.iter().into_internal().max(), Some(&3));
    /// assert_eq!(b.iter().into_internal().max(), None);
    /// ```
    fn max(self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        self.max_by(Ord::cmp)
    }

    /// Returns the maximum element of an iterator using a custom comparer
    /// function.
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

    /// Returns the minimum element of an iterator.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
    /// let a = [1, 2, 3];
    /// let b: Vec<u32> = Vec::new();
    ///
    /// assert_eq!(a.iter().into_internal().min(), Some(&1));
    /// assert_eq!(b.iter().into_internal().min(), None);
    /// ```
    fn min(self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        self.min_by(Ord::cmp)
    }

    /// Returns the minimum element of an iterator using a custom comparer
    /// function.
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

    /// Returns the first element of the iterator.
    ///
    /// Note that unlike [`Iterator::next`], this method consumes the iterator.
    /// It is really only useful for getting the first element in the iterator,
    /// and is called `next` just for api similarity with regular iterators.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
    /// let a = [1, 2, 3];
    /// assert_eq!(a.iter().into_internal().next(), Some(&1));
    /// ```
    fn next(self) -> Option<Self::Item> {
        self.find_map(Some)
    }

    /// Returns the `n`th element of the iterator.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
    /// let a = [1, 2, 3];
    /// assert_eq!(a.iter().into_internal().nth(1), Some(&2));
    /// ```
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

    /// Returns the index of the first element matching the predicate.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
    /// let a = [1, 2, 3];
    ///
    /// assert_eq!(a.iter().into_internal().position(|&x| x == 2), Some(1));
    ///
    /// assert_eq!(a.iter().into_internal().position(|&x| x == 5), None);
    /// ```
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

    /// Skip first `n` elements of the iterator.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
    /// let a = [1, 2, 3, 4];
    ///
    /// let v = a.iter().into_internal().skip(2).collect::<Vec<_>>();
    ///
    /// assert_eq!(v, vec![&3, &4]);
    /// ```
    fn skip(self, n: usize) -> Skip<Self> {
        Skip { iter: self, n }
    }

    // TODO: skip_while

    // TODO: step_by

    // TODO: sum

    /// Take first `n` elements of the iterator, disregarding the rest.
    ///
    /// ```
    /// # use internal_iterator::{InternalIterator, IteratorExt};
    /// let a = [1, 2, 3, 4];
    ///
    /// let v = a.iter().into_internal().take(2).collect::<Vec<_>>();
    ///
    /// assert_eq!(v, vec![&1, &2]);
    /// ```
    fn take(self, n: usize) -> Take<Self> {
        Take { iter: self, n }
    }

    // TODO: take_while

    // TODO: try_find

    // TODO: try_fold

    // TODO: try_for_each

    // TODO: unzip
}

/// Conversion to an [`InternalIterator`].
///
/// This is internal-iterator equivalent of [`std::iter::IntoIterator`].
pub trait IntoInternalIterator {
    /// The type of the elements being iterated over.
    type Item;
    /// Concrete iterator type returned by the conversion.
    type IntoIter: InternalIterator<Item = Self::Item>;

    /// Convert this type to an internal iterator.
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
    /// # use internal_iterator::InternalIterator;use internal_iterator::IteratorExt;
    ///
    /// fn flatten_ranges(
    ///     ranges: impl InternalIterator<Item = (i32, i32)>,
    /// ) -> impl InternalIterator<Item = i32> {
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

/// Conversion from an [`InternalIterator`].
///
/// This is internal-iterator equivalent of [`std::iter::FromIterator`].
pub trait FromInternalIterator<A> {
    /// Convert from an iterator.
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoInternalIterator<Item = A>;
}

impl<C, R, E> FromInternalIterator<Result<R, E>> for Result<C, E>
where
    C: FromInternalIterator<R>,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoInternalIterator<Item = Result<R, E>>
    {
        let mut error = None;
        let c = C::from_iter(iter
            .into_internal_iter()
            // FIXME: this could stop on first Err
            .filter_map(|r| match r {
                Ok(v) => Some(v),
                Err(e) => {
                    error = Some(e);
                    None
                }
            }));
        match error {
            Some(err) => Err(err),
            None => Ok(c),
        }
    }
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
