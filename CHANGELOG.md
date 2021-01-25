# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- Create an internal `_remove` method, which does not call to `balance`, avoiding unnecessary recursion.

## [0.1.2] - 03-12-20
### Added
- Balance the map on insert/remove.

## [0.1.1] - 30-11-20
### Added
- Implement `remove` method for Map.

## [0.1.0] - 14-11-20
### Added
- Initial implementation of map-like BST.
