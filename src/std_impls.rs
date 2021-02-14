use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;
use crate::FromInternalIterator;

impl<A> FromIternalIterator<A> for Vec<A> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoInternalIterator<Item = A>
    {
        let mut v = Vec::new();
        iter.into_internal_iter().for_each(|item| {
            v.push(item);
        });
        v
    }
}

impl FromIternalIterator<char> for String {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoInternalIterator<Item = A>
    {
        let mut v = String::new();
        iter.into_internal_iter().for_each(|item| {
            v.push(item);
        });
        v
    }
}

impl<T: Eq + Hash> FromIternalIterator<T> for HashSet<T> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoInternalIterator<Item = A>
    {
        let mut v = HashSet::new();
        iter.into_internal_iter().for_each(|item| {
            v.insert(item);
        });
        v
    }
}

impl<K: Eq + Hash, V> FromIternalIterator<(K, V)> for HashMap<K, V> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoInternalIterator<Item = A>
    {
        let mut v = HashMap::new();
        iter.into_internal_iter().for_each(|(k, v)| {
            v.insert(k, v);
        });
        v
    }
}

impl<T: Ord> FromIternalIterator<T> for BTreeSet<T> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoInternalIterator<Item = A>
    {
        let mut v = BTreeSet::new();
        iter.into_internal_iter().for_each(|item| {
            v.insert(item);
        });
        v
    }
}

impl<K: Ord, V> FromIternalIterator<(K, V)> for BTreeMap<K, V> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoInternalIterator<Item = A>
    {
        let mut v = BTreeMap::new();
        iter.into_internal_iter().for_each(|(k, v)| {
            v.insert(k, v);
        });
        v
    }
}
