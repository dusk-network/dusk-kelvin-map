# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Update canonical from `v0.5` to `v0.6` [#14](https://github.com/dusk-network/dusk-kelvin-map/issues/14)
- Deprecate the crate in favor of `dusk-hamt`

## [0.3.0] - 25-01-21
### Changed
- Create an internal `_remove` method, which does not call to `balance`, avoiding unnecessary recursion.
- Use microkelvin no-std allocator.

## [0.1.2] - 03-12-20
### Added
- Balance the map on insert/remove.

## [0.1.1] - 30-11-20
### Added
- Implement `remove` method for Map.

## [0.1.0] - 14-11-20
### Added
- Initial implementation of map-like BST.
