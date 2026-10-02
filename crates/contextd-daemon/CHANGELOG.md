# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/syntropd/contextd/compare/v0.3.1...v0.4.0) - 2026-10-02

### Added

- *(index)* implement semantic journal indexing and hybrid BM25 cosine search
- bump version to 0.3.2 for 5-frontier architecture plan

### Fixed

- *(index)* populate semantic journal index from journalctl on daemon startup and consolidate cosine similarity

### Other

- *(release)* bump package and dependency versions to v0.4.0
- apply cargo fmt across workspace
- format code with cargo fmt
