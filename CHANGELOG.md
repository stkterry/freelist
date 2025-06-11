# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
_

## [1.1.0] - 2025-06-11

### Added
- Workflows for test coverage and Miri validation
- `is_value` function to `Slot` for use in `Freelist::compactify`

### Changed
- Updated README.md to use badges provided by Workflows
- Iterators to use while loops for better test coverage
- warmup times on benchmarks from 8 seconds to 3 seconds



## [1.0.0] - 2025-05-31

### Added
- this CHANGELOG.md
- Function, tests, and documentation for `Freelist::compactify`
- Custom iterators (`IterFl`, `IterMutFl`, `IntoIterFl`) for `Freelist` slices, including tests
- Performance benchmarking for `Freelist` functions `iter` and `into_iter`
- Added module-level iterator function `size_hint`

### Changed
- Updated README.md
- Moved iterators to separate modules
- Doubled perfomance of functions `iter` and `iter_mut` and `into_iter` for `Freelist` (switched to custom iterators)
- Improved documentation for Freelist functions:
    - `push` (clarified that freelist returns the insertion index)
    - `next_available` (added examples)
    - `to_vec` (added examples)
    - `iter` (example now demonstrates skipping over empty slots)
    - `iter_mut` (example now demonstrates skipping over empty slots)
    - `index` (Notifies of possible panic conditions)
    - `index_mut` (Notifies of possible panic conditions)

### Removed
- FreelistIter struct (replaced by `IntoIterFl`)


## [0.9.4] - 2025-05-29

### Added
- LICENSE.md
- README.md badges
- Categories section to Cargo.toml

### Removed
- README.md links for sources

### Fixed
- README.md code-snippets now correctly identify as Rust code


## [0.9.3] - 2025-05-29
Yanked.  See [0.9.4]

## [0.9.2] - 2025-05-29

### Added

- Tests:
    - `with_capacity`
    - `to_vec`
    - `reserve`
    - `filled`
    - `size`
    - `free`
    - `get`
    - `get_mut`
    - `get_unchecked`
    - `get_unchecked_mut`
    - `iter`
    - `iter_mut`
    - `default`
    - `index`
    - `index_mut`
    - `from_vec`
    - `from_iter`
    - `double_ended_iter`

- Functions and documentation for Freelist:
    - `capacity`
    - `to_vec`
    - `with_capacity`

- Documentation for Freelist:
    - `new`
    - `default`
    - `index`
    - `index_mut`
    
### Changed
- Restructured project, separating `Freelist`, its IntoIter implementation, and `Slot`
- Improved README.md
- Expanded documentation for Freelist:
    - `push`
    - `next_available`
    - `remove`
    - `filled`
    - `size`
    - `free`
    - `get`
    - `get_mut`
    - `get_unchecked`
    - `get_unchecked_mut`
    - `iter`
    - `iter_mut`


## [0.9.1] - 2025-05-29

### Added
- New functions for `Slot` 
    - `to_some_unchecked`
    - `as_value_unchecked`
    - `as_value_unchecked_mut`

### Changed
- Struct `Container` changed name to `Slot`
- Field names for `Freelist`
- Improved `remove` function performance for `Freelist`

## [0.9.0] - 2025-05-22

### Initial Release

### Added
- Initial implementation details

### Changed
- `FreeList` became `Freelist`

### Removed
- Comparison implementation
- Removed comparison implementation from benchmarks

