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
