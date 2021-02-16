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
            for &x in &[1, 2, 3] {
                let result = f(x);
                if result.is_some() {
                    return result;
                }
            }
            // take(3) shouldn't expect any more items
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

#[test]
fn take_empty() {
    struct Iter;

    impl InternalIterator for Iter {
        type Item = i32;

        fn find_map<T, F>(self, _: F) -> Option<T>
        where
            F: FnMut(i32) -> Option<T>,
        {
            unreachable!()
        }
    }

    // We should not hit the unreachable panic.
    assert_eq!(Iter.take(0).next(), None);
}

#[cfg(feature = "alloc")]
#[test]
fn readme_example() {
    use alloc::vec;
    use alloc::vec::Vec;

    #[derive(Clone)]
    struct Tree(i32, Vec<Tree>);

    impl Tree {
        fn into_iter(self) -> TreeIter {
            TreeIter {
                tree: self,
                position: vec![0],
            }
        }

        fn into_internal_iter(self) -> TreeInternalIter {
            TreeInternalIter {
                tree: self,
            }
        }
    }

    struct TreeIter {
        tree: Tree,
        position: Vec<usize>,
    }

    // returns None if walking down the tree went out of bounds in child vector in
    // some level
    fn find_subtree<'a>(tree: &'a Tree, pos: &[usize]) -> Option<&'a Tree> {
        match pos {
            [] => Some(tree),
            [idx, rest @ ..] => {
                let child = &tree.1.get(*idx)?;
                find_subtree(child, rest)
            }
        }
    }

    impl Iterator for TreeIter {
        type Item = i32;

        fn next(&mut self) -> Option<i32> {
            loop {
                match self.position.as_slice() {
                    [0, rest @ ..] => {
                        let current_tree = find_subtree(&self.tree, &rest);
                        if let Some(tree) = current_tree {
                            let result = Some(tree.0);
                            if !tree.1.is_empty() {
                                // node has children, move position at first child
                                // of current position
                                self.position.push(0);
                            } else {
                                // node has no children, move position to next
                                // sibling
                                *self.position.last_mut().unwrap() += 1;
                            }
                            return result;
                        } else {
                            // current position is out of bounds - move up by one
                            // and advance to next sibling, then loop over to try
                            // again
                            self.position.pop();
                            *self.position.last_mut().unwrap() += 1;
                        }
                    }
                    [1] => {
                        return None;
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    struct TreeInternalIter {
        tree: Tree,
    }

    // we need a helper because we need to use `f` multiple times, but an arbitrary
    // FnMut cannot be copied or reborrowed
    fn find_map_helper<T>(tree: Tree, f: &mut impl FnMut(i32) -> Option<T>) -> Option<T> {
        let result = f(tree.0);
        if result.is_some() {
            return result;
        }
        for child in tree.1 {
            let result = find_map_helper(child, f);
            if result.is_some() {
                return result;
            }
        }
        None
    }

    impl InternalIterator for TreeInternalIter {
        type Item = i32;

        fn find_map<T, F>(self, mut f: F) -> Option<T>
        where
            F: FnMut(i32) -> Option<T>,
        {
            find_map_helper(self.tree, &mut f)
        }
    }

    let tree = Tree(4, vec![
        Tree(1, vec![]),
        Tree(3, vec![
            Tree(5, vec![]),
        ]),
        Tree(2, vec![]),
    ]);

    let iterator_result = tree
        .clone()
        .into_iter()
        .map(|x| x * 2)
        .filter(|&x| x > 5)
        .flat_map(|x| vec![x, x * 10])
        .collect::<Vec<_>>();

    assert_eq!(iterator_result, vec![8, 80, 6, 60, 10, 100]);

    let internal_iterator_result = tree
        .clone()
        .into_internal_iter()
        .map(|x| x * 2)
        .filter(|&x| x > 5)
        .flat_map(|x| vec![x, x * 10].into_iter().into_internal())
        .collect::<Vec<_>>();

    assert_eq!(internal_iterator_result, vec![8, 80, 6, 60, 10, 100]);
}
