use crate::*;

#[cfg(feature = "alloc")]
#[test]
fn take_short_circuit() {
    use alloc::vec;
    use alloc::vec::Vec;

    struct Iter<'a> {
        exhausted: &'a mut bool,
    }

    impl<'a> InternalIterator for Iter<'a> {
        type Item = i32;

        fn find_map<T, F>(self, mut f: F) -> Option<T>
        where
            F: FnMut(i32) -> Option<T>,
        {
            for &x in &[1, 2, 3, 4, 5] {
                let result = f(x);
                if result.is_some() {
                    return result;
                }
            }
            *self.exhausted = true;
            None
        }
    }

    let mut exhausted = false;
    let iter = Iter {
        exhausted: &mut exhausted,
    };

    let items = iter.take(3).collect::<Vec<_>>();
    assert_eq!(items, vec![1, 2, 3]);
    assert!(!exhausted);
}
