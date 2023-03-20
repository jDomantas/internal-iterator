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

        fn try_for_each<T, F>(self, mut f: F) -> ControlFlow<T>
        where
            F: FnMut(i32) -> ControlFlow<T>,
        {
            for &x in &[1, 2, 3] {
                f(x)?;
            }
            // take(3) shouldn't expect any more items
            *self.exhausted = true;
            ControlFlow::Continue(())
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

        fn try_for_each<T, F>(self, _: F) -> ControlFlow<T>
        where
            F: FnMut(i32) -> ControlFlow<T>,
        {
            unreachable!()
        }
    }

    // We should not hit the unreachable panic.
    assert_eq!(Iter.take(0).next(), None);
}

#[test]
fn map_side_effects_preserved() {
    let mut closure_calls = 0;
    let count = (10..25)
        .into_internal()
        .map(|x| {
            closure_calls += 1;
            x * 2
        })
        .count();
    assert_eq!(count, 15);
    assert_eq!(closure_calls, 15);

    let mut closure_calls = 0;
    let nth = (10..25)
        .into_internal()
        .map(|x| {
            closure_calls += 1;
            x * 2
        })
        .nth(12);
    assert_eq!(nth, Some(44));
    assert_eq!(closure_calls, 13);

    let mut closure_calls = 0;
    let last = (10..25)
        .into_internal()
        .map(|x| {
            closure_calls += 1;
            x * 2
        })
        .last();
    assert_eq!(last, Some(48));
    assert_eq!(closure_calls, 15);
}

#[test]
fn clone_side_effects_preserved() {
    use core::cell::Cell;

    struct Weird<'a>(i32, &'a Cell<i32>);
    impl Clone for Weird<'_> {
        fn clone(&self) -> Self {
            self.1.set(self.1.get() + 1);
            Weird(self.0, self.1)
        }
    }

    let clones = Cell::new(0);
    let weirds = [
        &Weird(0, &clones),
        &Weird(1, &clones),
        &Weird(2, &clones),
        &Weird(3, &clones),
    ];

    let count = weirds
        .into_internal_iter()
        .cloned()
        .count();
    assert_eq!(count, 4);
    assert_eq!(clones.get(), 4);
    clones.set(0);

    let nth = weirds
        .into_internal_iter()
        .cloned()
        .nth(2);
    assert_eq!(nth.unwrap().0, 2);
    assert_eq!(clones.get(), 3);
    clones.set(0);

    let last = weirds
        .into_internal_iter()
        .cloned()
        .last();
    assert_eq!(last.unwrap().0, 3);
    assert_eq!(clones.get(), 4);
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
    fn for_each_helper<T>(
        tree: Tree,
        f: &mut impl FnMut(i32) -> ControlFlow<T>,
    ) -> ControlFlow<T> {
        f(tree.0)?;
        for child in tree.1 {
            for_each_helper(child, f)?
        }
        ControlFlow::Continue(())
    }

    impl InternalIterator for TreeInternalIter {
        type Item = i32;

        fn try_for_each<T, F>(self, mut f: F) -> ControlFlow<T>
        where
            F: FnMut(i32) -> ControlFlow<T>,
        {
            for_each_helper(self.tree, &mut f)
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
        .flat_map(|x| [x, x * 10])
        .collect::<Vec<_>>();

    assert_eq!(iterator_result, vec![8, 80, 6, 60, 10, 100]);

    let internal_iterator_result = tree
        .clone()
        .into_internal_iter()
        .map(|x| x * 2)
        .filter(|&x| x > 5)
        .flat_map(|x| [x, x * 10])
        .collect::<Vec<_>>();

    assert_eq!(internal_iterator_result, vec![8, 80, 6, 60, 10, 100]);
}
