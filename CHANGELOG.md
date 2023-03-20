# 0.2.1

- Adapters now derive `Clone`, just like std iterators [#11](https://github.com/jDomantas/internal-iterator/pull/11)
- Some adapters now specialize `count`, `nth`, and `last` to be more efficient [#11](https://github.com/jDomantas/internal-iterator/pull/11) [#12](https://github.com/jDomantas/internal-iterator/pull/12)

# 0.2.0

- Make `try_for_each` the primary method [#8](https://github.com/jDomantas/internal-iterator/pull/8)
- Add `IntoInternalIterator` impls for builtin collections [#7](https://github.com/jDomantas/internal-iterator/pull/7)

# 0.1.2

- Added `max_by_key` and `min_by_key` [#4](https://github.com/jDomantas/internal-iterator/pull/4)

# 0.1.1

- `.take(n)` will not consume `n+1`-th element of the underlying iterator [#2](https://github.com/jDomantas/internal-iterator/pull/2)
- Docs improvements

# 0.1.0

Initial release.
