# Internal Iterator

[![Crates.io](https://img.shields.io/crates/v/internal-iterator.svg)](https://crates.io/crates/internal-iterator)
[![API reference](https://docs.rs/internal-iterator/badge.svg)](https://docs.rs/internal-iterator/)
[![License](https://img.shields.io/badge/license-MIT_OR_Apache--2.0-blue.svg)](
https://github.com/jDomantas/internal-iterator#license)
[![Tests](https://github.com/jDomantas/internal-iterator/workflows/Tests/badge.svg)](https://github.com/jDomantas/internal-iterator/actions?query=workflow%3ATests+branch%3Amaster)

Internal iterator equivalent of `std::iter::Iterator`.

Featuring:

* `std`-like api
* `#![forbid(unsafe)]`
* zero dependencies
* optional dependency on `std` and `alloc` (enabled by default)

## Motivation

The difference between external and internal iteration is:

* With external iteration, you call `next()` repeatedly to get elements out of
the iterator. Iterators must store its iteration state to know where to pick up
on each `next` call. External iteration gives more power to the consumer - for
example, it is very easy to implement a `for` loop by transforming it to
`while let Some(item) = iter.next() { ... }`.
* With internal iteration, you provide a closure and the iterator calls it with
every element it wants to yield. This is similar to what `Iterator::for_each`
would do. Iterators don't need to store the iteration state explicitly which
simplifies iterator implementations for some data structures. Internal iteration
gives more power for the iterator.

One example where internal iterators shine (and the original reason for this crate) is iteration over trees. Let's take a very simple tree of integers:

```rust
struct Tree(i32, Vec<Tree>);
```

To be able to implement `next()` we need to store the current position in the
tree. We could represent this as a list of indices, each one indicating a child
on the subtree in corresponding level:

```rust
struct TreeIter {
    tree: Tree,
    position: Vec<usize>,
}
```

The `Iterator` implementation would yield the value at the current position, and
advance either to the subtree or the next sibling, climbing up

```rust
// returns None if walking down the tree went out of bounds in child vector in
// some level
fn find_subtree<'a>(tree: &'a Tree, pos: &[usize]) -> Option<&'a Tree> {
    match pos {
        [] => tree,
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
                    let current_tree = find_subtree(&self.tree, &self.position);
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
                        return Some(result);
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
```

Whew, that was quite tricky, with all the index jugling and all.

Let's try the same with `InternalIterator`. The core method that drives the
trait is `find_map`:

```rust
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
```

That was a lot more straightforward, less error prone, and does not even require
any dynamic memory allocation!

Both of them allow constructing elaborate iterator pipelines:

```rust
let tree = Tree(4, vec![
    Tree(1, vec![]),
    Tree(3, vec![
        Tree(5, vec![]),
    ]),
    Tree(2, vec![]),
]);

let iterator_result = tree
    .into_iter()
    .map(|x| x * 2)
    .filter(|&x| x > 5)
    .flat_map(|x| [x, x * 10])
    .collect::<Vec<_>>();

assert_eq!(iterator_result, vec![8, 80, 6, 60, 10, 100]);

let internal_iterator_result = tree
    .into_internal_iter()
    .map(|x| x * 2)
    .filter(|&x| x > 5)
    .flat_map(|x| [x, x * 10].into_iter().into_internal())
    .collect::<Vec<_>>();

assert_eq!(internal_iterator_result, vec![8, 80, 6, 60, 10, 100]);
```

Internal iterators are not as expressive as external ones - they cannot be used
with `for` loops or with some other adaptors that require external iteration.
However, the loss of of expressiveness is not that big, and depending on your
use-case might be a small price to pay for a far simpler iterator
implementations.

## Limitations

This crate aims to provide a straightforward api that is very similar to one in
`std`, which means that some fancy features that could be provided by internal
iteration are not available in this library.

## About missing `Iterator` methods

Not all method equivalents from `std::iter::Iterator` are implemented. Some of
those are impossible (`zip` is one of those), while most of the others are not
implemented just because I didn't personally need them yet.

If you see value in this library but some of the methods you need are missing
feel free to open an issue or submit a pull request.

## License

Licensed under either of

* Apache License, Version 2.0
    ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
    ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
