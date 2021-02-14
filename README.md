# Internal Iterator

Internal iterator equivalent of `std::iter::Iterator`.

Features:

* `std`-like api
* `#![forbid(unsafe)]`
* zero dependencies
* optional dependency on `std` and `alloc` (enabled by default)

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
