# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.2] - 2026-01-06

### Fixed
- Fixed Clippy warning `only_used_in_recursion` in FP-Growth algorithm
  - Changed `collect_paths_for_item()` and `count_items_recursive()` from instance methods to associated functions
- Fixed Clippy warnings in example files
  - Replaced `.or_insert_with(Vec::new)` with `.or_default()` in multiple examples
  - Removed unused imports and variables
  - Fixed unused mutable variable declarations
- Fixed doctest in `RuleMiner::add_transactions_from_iter()` to include required `ColumnMapping` parameter
- Applied code formatting across all files to meet style guidelines

### Improved
- All CI checks now pass with strict warnings (`-D warnings`)
- Better code quality and consistency across the codebase

## [0.2.1] - Previous Release

### Added
- Excel/CSV data loading with ColumnMapping
- PostgreSQL streaming support
- Cloud storage support (S3, HTTP)
- Engine integration with rust-rule-engine
- GRL export format
- Sequential pattern mining
- Graph-based pattern mining

## [0.2.0] - Previous Release

### Added
- Initial release with association rule mining
- Apriori and FP-Growth algorithms
- Quality metrics (confidence, support, lift, conviction)
- Transaction-based data model
