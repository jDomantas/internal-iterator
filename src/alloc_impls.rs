use alloc::{string::String, vec::Vec, collections::{BTreeMap, BTreeSet}};
use crate::{FromInternalIterator, InternalIterator, IntoInternalIterator, IteratorExt};

impl<A> FromInternalIterator<A> for Vec<A> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoInternalIterator<Item = A>
    {
        let mut result = Vec::new();
        iter.into_internal_iter().for_each(|item| {
            result.push(item);
        });
        result
    }
}

impl FromInternalIterator<char> for String {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoInternalIterator<Item = char>
    {
        let mut result = String::new();
        iter.into_internal_iter().for_each(|item| {
            result.push(item);
        });
        result
    }
}

impl<A: Ord> FromInternalIterator<A> for BTreeSet<A> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoInternalIterator<Item = A>
    {
        let mut result = BTreeSet::new();
        iter.into_internal_iter().for_each(|item| {
            result.insert(item);
        });
        result
    }
}

impl<K: Ord, V> FromInternalIterator<(K, V)> for BTreeMap<K, V> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoInternalIterator<Item = (K, V)>
    {
        let mut result = BTreeMap::new();
        iter.into_internal_iter().for_each(|(k, v)| {
            result.insert(k, v);
        });
        result
    }
}

crate::into_internal_impls! {
    ['a, T] &'a Vec<T>,
    ['a, T] &'a mut Vec<T>,
    [T] Vec<T>,
    ['a, T] &'a BTreeSet<T>,
    [T] BTreeSet<T>,
    ['a, K, V] &'a BTreeMap<K, V>,
    ['a, K, V] &'a mut BTreeMap<K, V>,
    [K, V] BTreeMap<K, V>,
}
