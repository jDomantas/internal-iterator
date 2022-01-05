use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use crate::{FromInternalIterator, InternalIterator, IntoInternalIterator, IteratorExt};

impl<A: Eq + Hash> FromInternalIterator<A> for HashSet<A> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoInternalIterator<Item = A>
    {
        let mut result = HashSet::new();
        iter.into_internal_iter().for_each(|item| {
            result.insert(item);
        });
        result
    }
}

impl<K: Eq + Hash, V> FromInternalIterator<(K, V)> for HashMap<K, V> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoInternalIterator<Item = (K, V)>
    {
        let mut result = HashMap::new();
        iter.into_internal_iter().for_each(|(k, v)| {
            result.insert(k, v);
        });
        result
    }
}

crate::into_internal_impls! {
    ['a, T] &'a HashSet<T>,
    [T] HashSet<T>,
    ['a, K, V] &'a HashMap<K, V>,
    ['a, K, V] &'a mut HashMap<K, V>,
    [K, V] HashMap<K, V>,
}
