use core::marker::PhantomData;
use super::*;

/// An iterator returned by [`from_fn`].
pub struct FromFn<F, R> {
    f: F,
    _marker: PhantomData<fn() -> R>,
}

/// A helper type used in [`from_fn`].
///
/// It represents a value that the iterator is stopped with, which `from_fn`
/// closure needs to pass back to the caller. It has a lifetime parameter to
/// prevent values from different scopes from getting mixed up (i.e. if
/// `from_fn` is used within another `from_fn`).
pub struct BreakValue<'a> {
    _invariant: PhantomData<fn(&'a ()) -> &'a ()>,
}

impl<F, R> InternalIterator for FromFn<F, R>
where
    F: for<'a> FnOnce(&mut dyn FnMut(R) -> ControlFlow<BreakValue<'a>>) -> ControlFlow<BreakValue<'a>>,
{
    type Item = R;

    fn try_for_each<RR, FF>(self, mut f: FF) -> ControlFlow<RR>
    where
        FF: FnMut(Self::Item) -> ControlFlow<RR>,
    {
        let mut result = ControlFlow::Continue(());
        (self.f)(&mut |item| {
            match f(item) {
                ControlFlow::Continue(()) => ControlFlow::Continue(()),
                ControlFlow::Break(res) => {
                    result = ControlFlow::Break(res);
                    ControlFlow::Break(BreakValue { _invariant: PhantomData })
                }
            }
        });
        result
    }
}

/// Creates an internal iterator from provided closure.
///
/// Provided closure should be equivalent to how
/// [`InternalIterator::try_for_each`] would be implemented - it should call its
/// parameter with every value that needs to be yielded from the iterator,
/// respect returned `ControlFlow` value, and return
/// `ControlFlow::Continue(())` at the end. Type signature ensures most of
/// that, you only need to take care to use `?` operator on every call when
/// yielding values.
///
/// If you want to construct an [`InternalIterator`] from a closure that yields
/// items one by one, like you would with [`std::iter::from_fn`], then you can
/// use that function with a conversion to internal iterator:
/// `std::iter::from_fn(f).into_internal()`.
///
/// ```
/// # use internal_iterator::InternalIterator;
/// # use std::ops::ControlFlow;
/// let x = 2;
/// let y = Some(3);
///
/// let iter = internal_iterator::from_fn(|f| {
///     f(1)?;
///     f(x)?;
///     if let Some(value) = y {
///         f(value)?;
///     }
///     ControlFlow::Continue(())
/// });
///
/// let values = iter.collect::<Vec<_>>();
/// assert_eq!(values, [1, 2, 3])
/// ```
///
/// Note that [`InternalIterator::try_for_each`] function is generic, but
/// generic closures are not possible. Therefore this function utilizes dynamic
/// dispatch (to be able to provide any function to the closure), and a marker
/// type for break value with the actual value passed via a side channel
/// (to be able to use any type as break value). Because of this, iterators
/// constructed by [`from_fn`] might be optimized more poorly. If the need
/// arises such iterators can always be rewritten as explicit structs with
/// generic implementations of [`InternalIterator::try_for_each`], although that
/// will require manually handling captured variables (whereas compiler does
/// that for you when using closures).
pub fn from_fn<F, R>(f: F) -> FromFn<F, R>
where
    F: for<'a> FnOnce(&mut dyn FnMut(R) -> ControlFlow<BreakValue<'a>>) -> ControlFlow<BreakValue<'a>>,
{
    FromFn { f, _marker: PhantomData }
}

#[test]
fn pipeline_tests() {
    fn check(iter: impl InternalIterator<Item = i32>, expect: &[i32]) {
        assert_eq!(iter.collect::<Vec<_>>(), expect);
    }
    fn make_iter() -> impl InternalIterator<Item = i32> {
        from_fn(|f| {
            f(1)?;
            f(2)?;
            f(3)?;
            f(4)?;
            ControlFlow::Continue(())
        })
    }
    check(make_iter(), &[1, 2, 3, 4]);
    check(make_iter().map(|x| x * 2), &[2, 4, 6, 8]);
    check(make_iter().filter(|&x| x > 2), &[3, 4]);
    check(make_iter().filter(|&x| x <= 2), &[1, 2]);
    check(make_iter().chain([7, 8, 9]), &[1, 2, 3, 4, 7, 8, 9]);
}

#[test]
fn can_be_static() {
    fn use_static_iterator(iter: impl InternalIterator<Item = i32> + 'static) {
        assert_eq!(iter.collect::<Vec<_>>(), &[1, 2, 3]);
    }
    let mut a = 1;
    let iter = from_fn(move |f| {
        f(a)?;
        a += 1;
        f(a)?;
        a += 1;
        f(a)?;
        ControlFlow::Continue(())
    });
    use_static_iterator(iter);
}

/// ```compile_fail
/// use std::ops::ControlFlow;
/// use internal_iterator::InternalIterator;
///
/// internal_iterator::from_fn(|f| {
///     f(1)?;
///     let mut slot = ControlFlow::Continue(());
///     internal_iterator::from_fn(|f2| {
///         slot = f2(42); // this assignment is supposed to fail to compile,
///                        // BreakValue can be moved between from_fn invocations
///         ControlFlow::Continue(())
///     }).collect::<Vec<_>>();
///     slot
/// });
/// ```
fn __compile_fail_check() {}
